use std::cell::RefCell;
use std::collections::HashMap;

use samp::amx::{AmxIdent, get as get_amx};

use crate::state::{HttpsResponse, drain_responses};

thread_local! {
    /// Headers of the response currently being delivered to a Pawn public.
    /// Set immediately before `amx.exec` and cleared right after, so any
    /// native invoked from inside the callback can read it via `current_header`.
    static CURRENT_HEADERS: RefCell<Option<HashMap<String, String>>> = const { RefCell::new(None) };
}

pub fn print_banner() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let repository = env!("CARGO_PKG_REPOSITORY");
    let build_date = env!("BUILD_DATE");
    let build_time = env!("BUILD_TIME");
    let build_year = env!("BUILD_YEAR");

    samp::log::info!("");
    samp::log::info!("  | {} {} | {}", name, version, build_year);
    samp::log::info!("  |-------------------------------");
    samp::log::info!(
        "  | Author and maintainer: {}",
        value_or(authors, "Unknown")
    );
    samp::log::info!("");
    samp::log::info!("  | Compiled: {} at {}", build_date, build_time);
    samp::log::info!("  |-------------------------------");
    samp::log::info!("  | Repository: {}", value_or(repository, "N/A"));
    samp::log::info!("");
}

fn value_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

/// Returns the value of a response header (case-insensitive name lookup) for
/// the response currently being dispatched to its Pawn callback. Returns
/// `None` when called outside of a callback or when the header is absent.
pub fn current_header(key: &str) -> Option<String> {
    let lookup = key.to_ascii_lowercase();
    CURRENT_HEADERS.with(|cell| cell.borrow().as_ref().and_then(|h| h.get(&lookup).cloned()))
}

/// Drains up to `max` responses from the queue and dispatches each Pawn
/// callback by searching for the public across every loaded AMX.
pub fn dispatch_responses(amx_list: &[AmxIdent], max: usize) {
    for item in drain_responses(max) {
        dispatch_one(amx_list, item);
    }
}

fn dispatch_one(amx_list: &[AmxIdent], item: HttpsResponse) {
    if crate::state::take_cancelled(item.index) {
        return;
    }
    CURRENT_HEADERS.with(|cell| *cell.borrow_mut() = Some(item.headers.clone()));

    let mut delivered = false;
    for ident in amx_list {
        let Some(amx) = get_amx(*ident) else { continue };
        let Ok(func) = amx.find_public(&item.callback) else {
            continue;
        };

        let allocator = amx.allocator();
        let Ok(amx_str) = allocator.allot_string(&item.response) else {
            continue;
        };

        if amx.push(item.error).is_err()
            || amx.push(item.status).is_err()
            || amx.push(amx_str).is_err()
            || amx.push(item.index).is_err()
        {
            continue;
        }
        let _ = amx.exec(func);
        delivered = true;
        break;
    }

    CURRENT_HEADERS.with(|cell| *cell.borrow_mut() = None);

    if !delivered {
        crate::warn!("callback '{}' not found in any AMX", item.callback);
    }
}
