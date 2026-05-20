// rest-methods.pwn — PUT, DELETE, and PATCH against a REST resource.
//
// Demonstrates:
//   * Using HTTPS_PUT / HTTPS_DELETE / HTTPS_PATCH.
//   * Reading the HTTP status to confirm idempotent updates.

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    // PUT: full replacement
    https_jsonf("{\"name\":\"erick\",\"score\":100}");
    https(1, HTTPS_PUT, "https://httpbin.org/anything/players/42", "", "OnRest");

    // PATCH: partial update
    https_jsonf("{\"score\":150}");
    https(2, HTTPS_PATCH, "https://httpbin.org/anything/players/42", "", "OnRest");

    // DELETE: no body
    https(3, HTTPS_DELETE, "https://httpbin.org/anything/players/42", "", "OnRest");
    return 1;
}

forward OnRest(index, response[], status, error);
public  OnRest(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] req %d failed: error=%d", index, error);
        return 1;
    }
    printf("[example] req %d status=%d", index, status);
    return 1;
}
