# Cookies

Cookie persistence is disabled by default. APIs that authenticate through HTTP sessions, or any endpoint that expects the client to honor `Set-Cookie`, require the cookie store to be turned on explicitly.

## Enabling the cookie store

`https_cookies_enable(true)` activates an in-memory cookie jar. From that point on, every cookie received in a `Set-Cookie` header is parsed, stored, and automatically re-sent on subsequent requests to the same origin, respecting standard cookie scoping rules (host, path, expiration). Calling the native with `false` disables the store; existing cookies are not deleted, they simply stop being sent and no new ones are recorded.

The toggle is global, not per-request. Every request issued while the store is enabled participates in the same jar.

## Clearing the jar

`https_cookies_clear()` replaces the jar with a fresh, empty one. Subsequent requests start clean, regardless of whether the store is currently enabled. This is useful for tests, for logging out a session, or for switching identities.

The clear operation is immediate and unconditional. There is no facility to clear cookies for a specific host or path.

## Interaction with other features

- **Redirects.** Cookies received during a redirect chain are stored as they arrive and applied to follow-up requests in the same chain, exactly the same way a regular browser would handle them.
- **mTLS.** The cookie jar is shared between the default client and the mTLS-enabled client. Cookies persisted while mTLS was active will be sent on subsequent non-mTLS requests to the same origin, and vice versa.
- **Cross-host redirects.** When a redirect crosses to a different host, cookies scoped to the original host are not forwarded to the new host — that is a property of how cookies work, not a plugin policy. The `Authorization` stripping described in [Security](security.md) applies independently.

## Lifetime and limits

The jar lives for as long as the plugin is loaded. There is no on-disk persistence; restarting the server starts with an empty jar. There is no explicit cap on the number of stored cookies — under realistic gamemode usage the jar size is bounded by what the server has actually been talking to, which is small.

The store is not exposed to Pawn for inspection. If a particular cookie value needs to be visible to script code, read it from the `Set-Cookie` response header inside the relevant callback using `https_response_header("set-cookie", ...)` and parse it manually.
