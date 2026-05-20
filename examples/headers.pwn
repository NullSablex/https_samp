// headers.pwn — global headers and per-request overrides.
//
// Demonstrates:
//   * Installing a global header that survives across requests.
//   * Adding a temporary header that applies only to the next call.
//   * Overriding the global value for one specific request via the temporary layer.

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    // Set once; reused on every subsequent request until cleared.
    https_set_global_header("Authorization", "Bearer demo-token");
    https_set_global_header("Accept", "application/json");

    // First request: uses the global Authorization above.
    https(1, HTTPS_GET, "https://httpbin.org/headers", "", "OnHeadersDone");

    // Second request: overrides Authorization just for this call.
    https_set_header("Authorization", "Bearer other-token");
    https(2, HTTPS_GET, "https://httpbin.org/headers", "", "OnHeadersDone");
    return 1;
}

forward OnHeadersDone(index, response[], status, error);
public  OnHeadersDone(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] req %d failed: error=%d", index, error);
        return 1;
    }
    printf("[example] req %d ok: status=%d", index, status);
    return 1;
}
