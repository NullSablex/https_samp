// cross-host-redirect.pwn — explicitly allow a redirect to a different host.
//
// Demonstrates:
//   * The one-shot cross-host opt-in flag.
//   * That the flag is consumed at submission time (no persistence).
//   * That POLICY_BLOCKED is returned when the flag is not set.

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    // Without the opt-in, a redirect that crosses hosts is refused with
    // HTTPS_ERROR_POLICY_BLOCKED. Enable it for this single request only.
    https_allow_cross_host_once(true);
    https(1, HTTPS_GET, "https://httpbin.org/redirect-to?url=https://example.com/", "", "OnRedirectDone");

    // The next request below does NOT carry the opt-in any more — the flag
    // has been consumed. A cross-host redirect here would be blocked.
    https(2, HTTPS_GET, "https://example.com/", "", "OnRedirectDone");
    return 1;
}

forward OnRedirectDone(index, response[], status, error);
public  OnRedirectDone(index, response[], status, error)
{
    if (error == HTTPS_ERROR_POLICY_BLOCKED)
    {
        printf("[example] req %d blocked by policy", index);
        return 1;
    }
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] req %d failed: error=%d", index, error);
        return 1;
    }
    printf("[example] req %d ok: status=%d", index, status);
    return 1;
}
