// post-json.pwn — POST a JSON body using the JSON builder.
//
// Demonstrates:
//   * Staging a JSON payload with https_jsonf (Content-Type is set automatically).
//   * Submitting a POST with an empty inline body so the staged payload is consumed.
//   * A custom header that overrides the builder's default Content-Type if needed.

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    // The JSON is validated before being staged.
    if (!https_jsonf("{\"player\":\"erick\",\"score\":42}"))
    {
        print("[example] invalid JSON, request not sent");
        return 1;
    }

    // Optional: a temporary header just for this request.
    https_set_header("X-Trace-Id", "example-001");

    // Empty "data" tells the plugin to use the staged payload.
    https(1, HTTPS_POST, "https://httpbin.org/post", "", "OnPostJson");
    return 1;
}

forward OnPostJson(index, response[], status, error);
public  OnPostJson(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] post %d failed: error=%d", index, error);
        return 1;
    }

    printf("[example] post %d ok: status=%d, %d bytes", index, status, strlen(response));
    return 1;
}
