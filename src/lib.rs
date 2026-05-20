use samp::amx::AmxIdent;
use samp::initialize_plugin;
use samp::plugin::TickContext;
use samp::prelude::*;

mod https;
#[macro_use]
mod logger;
mod natives;
mod state;
mod util;

const RESPONSES_PER_TICK: usize = 64;

pub struct Plugin {
    amx_list: Vec<AmxIdent>,
}

impl Plugin {
    fn new() -> Self {
        Self { amx_list: Vec::new() }
    }
}

impl SampPlugin for Plugin {
    fn on_load(&mut self) {
        util::print_banner();
        if samp::plugin::omp_core().is_some() {
            info!("running on native Open Multiplayer");
        } else {
            info!("running on SA-MP (or Open Multiplayer legacy mode)");
        }
    }

    fn on_unload(&mut self) {
        info!("unloaded");
    }

    fn on_amx_load(&mut self, amx: &Amx) {
        self.amx_list.push(amx.ident());
    }

    fn on_amx_unload(&mut self, amx: &Amx) {
        let ident = amx.ident();
        self.amx_list.retain(|id| *id != ident);
    }

    /// Unified tick: fires on SA-MP (ProcessTick) and on native Open Multiplayer
    /// (ITimersComponent at 5 ms). Drains pending responses and dispatches the
    /// Pawn callbacks — no Pawn timer required.
    fn on_tick(&mut self, _ctx: TickContext) {
        util::dispatch_responses(&self.amx_list, RESPONSES_PER_TICK);
    }

    fn on_omp_ready(&mut self) {
        info!("Open Multiplayer: every component ready");
    }

    fn on_component_free(&mut self) {
        info!("Open Multiplayer: a component was released");
    }
}

initialize_plugin!(
    natives: [
        Plugin::https,
        Plugin::https_set_header,
        Plugin::https_set_global_header,
        Plugin::https_clear_global_headers,
        Plugin::https_response_header,
        Plugin::https_process_queue,
        Plugin::https_set_max_body_bytes,
        Plugin::https_get_max_body_bytes,
        Plugin::https_queue_len,
        Plugin::https_allow_cross_host_once,
        Plugin::https_set_timeout_once,
        Plugin::https_cancel,
        Plugin::https_bodyf,
        Plugin::https_jsonf,
        Plugin::https_form_add,
        Plugin::https_multipart_add_text,
        Plugin::https_multipart_add_file,
        Plugin::https_set_basic_auth_once,
        Plugin::https_set_bearer_once,
        Plugin::https_cookies_enable,
        Plugin::https_cookies_clear,
        Plugin::https_mtls_set_pem,
        Plugin::https_mtls_set_pem_file,
        Plugin::https_mtls_clear,
    ],
    {
        samp::plugin::enable_tick();
        return Plugin::new();
    }
);
