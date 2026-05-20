// simple-get.pwn — fire a GET request and print the response.
//
// Demonstrates:
//   * The minimal request shape.
//   * The four-argument callback signature.
//   * Inspecting the error code before reading the body.

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    // The "1" here is an arbitrary correlation id; the plugin echoes it back.
    https(1, HTTPS_GET, "https://httpbin.org/get", "", "OnSimpleGet");
    return 1;
}

forward OnSimpleGet(index, response[], status, error);
public  OnSimpleGet(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] request %d failed: error=%d", index, error);
        return 1;
    }

    printf("[example] request %d ok: status=%d, %d bytes", index, status, strlen(response));
    printf("[example] body: %s", response);
    return 1;
}
