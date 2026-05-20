// cookies-session.pwn — login then call protected endpoints sharing the session cookie.
//
// Demonstrates:
//   * Enabling the cookie store before issuing the login request.
//   * Subsequent requests automatically reusing the session cookie.
//   * Clearing cookies when logging out.

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    https_cookies_enable(true);

    // Login: the response Set-Cookie is stored in the jar automatically.
    https_form_add("username", "erick");
    https_form_add("password", "s3cret");
    https(1, HTTPS_POST, "https://httpbin.org/cookies/set/session/abc123", "", "OnLogin");
    return 1;
}

forward OnLogin(index, response[], status, error);
public  OnLogin(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] login failed: error=%d", error);
        return 1;
    }
    // The session cookie is now stored — the next call sends it automatically.
    https(2, HTTPS_GET, "https://httpbin.org/cookies", "", "OnSessionCheck");
    return 1;
}

forward OnSessionCheck(index, response[], status, error);
public  OnSessionCheck(index, response[], status, error)
{
    if (error == HTTPS_ERROR_NONE)
    {
        printf("[example] session check status=%d body=%s", status, response);
    }
    // Logout flow: wipe the jar.
    https_cookies_clear();
    return 1;
}
