// auth-helpers.pwn — Basic and Bearer auth helpers.
//
// Demonstrates:
//   * https_set_basic_auth_once and https_set_bearer_once.
//   * That both are one-shot (cleared after the next request).

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    // Basic auth — the plugin base64-encodes "user:password" automatically.
    https_set_basic_auth_once("admin", "s3cret");
    https(1, HTTPS_GET, "https://httpbin.org/basic-auth/admin/s3cret", "", "OnAuth");

    // Bearer token for a different endpoint.
    https_set_bearer_once("eyJhbGciOiJIUzI1NiJ9...");
    https(2, HTTPS_GET, "https://httpbin.org/bearer", "", "OnAuth");
    return 1;
}

forward OnAuth(index, response[], status, error);
public  OnAuth(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] auth req %d failed: error=%d", index, error);
        return 1;
    }
    printf("[example] auth req %d ok: status=%d", index, status);
    return 1;
}
