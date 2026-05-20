# Body builders

POST requests can carry their payload in two ways: inline through the `data` argument of the `https` native, or staged ahead of time using one of three builders. The builders exist to keep call sites readable when the body is structured (JSON, form data) or large enough that embedding it in the call would obscure the request.

## How the staging works

The plugin keeps a single one-shot payload slot. A successful builder call writes into that slot, replacing whatever was there previously. The slot is consumed by the next POST issued through `https` whose inline `data` argument is empty: the staged bytes become the request body, and the corresponding `Content-Type` (when the builder implies one) is moved into the temporary header table.

If the next POST passes a non-empty `data`, that inline body wins and the staged payload is left untouched, waiting for a subsequent POST with empty `data`. If a GET or HEAD is issued in the meantime, the staged payload is also left untouched — only POST consumes it.

This design means each request has exactly one body source, chosen unambiguously: inline if supplied, otherwise the staged one. There is no implicit merging.

## Raw body

`https_bodyf` stages a verbatim byte sequence. No validation is performed beyond the size cap (the staged payload must not exceed the current value of `https_get_max_body_bytes`). The builder does not set `Content-Type`; if the body is not plain text, supply one via `https_set_header` before dispatching the request.

A successful call clears any previously staged JSON or form payload, so the three builders are mutually exclusive — the last one to succeed wins.

## JSON body

`https_jsonf` stages a string after validating that it parses as JSON. Invalid JSON is rejected and the slot is not modified, so calling the native and inspecting its return value is enough to know whether the payload is well-formed. The validator is the standard `serde_json` parser; anything it accepts is accepted here.

On success the builder also installs `Content-Type: application/json; charset=utf-8` in the temporary header table. If the remote service expects a different media type (`application/vnd.api+json`, for example), override it with `https_set_header` after the builder call.

The size cap applies to the bytes of the JSON string as supplied — the plugin does not re-serialize or compact it.

## Multipart/form-data body

`https_multipart_add_text` appends a text field and `https_multipart_add_file` appends a file field to a builder that accumulates `multipart/form-data` parts across calls. Files are referenced by their path and opened by the worker thread at send time; the append native only validates that the path exists at submission time.

Unlike the other builders, multipart does not stage a single byte sequence — it stages an opaque structure that the worker turns into a properly formatted multipart body, picks a unique boundary, and sets the `Content-Type` automatically (including the boundary). For this reason, multipart payloads cannot be replayed across redirects: a 307 or 308 redirect that would normally preserve the body discards a multipart payload and the second attempt sends no body. Avoid multipart against endpoints known to redirect.

The multipart builder is mutually exclusive with the other builders. The first multipart append clears the raw/json/form slots, and a successful raw/json/form call clears the multipart accumulator.

## Form-urlencoded body

`https_form_add` appends one key/value pair to a builder that accumulates `application/x-www-form-urlencoded` pairs across calls. Each successful append re-serializes the entire set, applies the size cap to the result, and stages it as the body. The `Content-Type` is installed as a temporary header on every successful append, so it is always consistent with the body when the request is dispatched.

Special characters in keys and values are percent-encoded according to the form-urlencoded rules; you supply the raw strings. Adding the same key twice appends a second occurrence rather than replacing the first, which matches what `application/x-www-form-urlencoded` actually means.

A successful append clears any previously staged raw or JSON payload, so switching styles between requests is safe.

## When to use which

- **Inline `data`.** Small, opaque payloads. The body fits comfortably on the call site and is not structured.
- **`https_bodyf`.** Large or binary-shaped payloads. Anything where you already have the final bytes and just need a way to ship them without inlining.
- **`https_jsonf`.** Structured JSON where you want the plugin to catch obvious formatting mistakes before the request hits the wire.
- **`https_form_add`.** Form submissions, classic API endpoints expecting query-style pairs. The accumulator makes it natural to build the body across several lines.
- **`https_multipart_add_text` / `https_multipart_add_file`.** Endpoints that expect `multipart/form-data` — file uploads, mixed text/binary fields, classic HTML form submissions with `enctype="multipart/form-data"`.

## Lifetime of the staged payload

The staged payload survives across requests if it is never consumed. Submitting a GET, HEAD, or a POST with non-empty inline `data` does not touch it. The only events that clear the slot are:

- A successful POST with empty inline `data` (the slot is taken and applied).
- A successful call to any of the three builders (replaces the slot).

The plugin does not expose an explicit "clear staged body" native. To discard the staged payload without sending it, either call a builder with trivial content and then never send a POST, or simply submit the next POST with the desired inline body and accept that the staged one will linger until consumed.
