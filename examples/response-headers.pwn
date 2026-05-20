// response-headers.pwn — read response headers inside the callback.
//
// Demonstrates:
//   * Using https_response_header from inside the callback.
//   * Reading common headers (rate-limit, ETag).

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    https(1, HTTPS_GET, "https://api.github.com/zen", "", "OnZen");
    return 1;
}

forward OnZen(index, response[], status, error);
public  OnZen(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] zen failed: error=%d", error);
        return 1;
    }

    new etag[128], remaining[16];
    if (https_response_header("ETag", etag))
    {
        printf("[example] etag: %s", etag);
    }
    if (https_response_header("X-RateLimit-Remaining", remaining))
    {
        printf("[example] rate-limit remaining: %s", remaining);
    }
    printf("[example] status=%d body=%s", status, response);
    return 1;
}
