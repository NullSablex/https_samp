// form-post.pwn — build an application/x-www-form-urlencoded payload.
//
// Demonstrates:
//   * Accumulating key/value pairs across multiple https_form_add calls.
//   * The builder setting Content-Type automatically.
//   * Sending a POST whose body comes entirely from the staged form.

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    https_form_add("username", "erick");
    https_form_add("password", "s3cret");
    https_form_add("remember", "1");

    https(1, HTTPS_POST, "https://httpbin.org/post", "", "OnFormPost");
    return 1;
}

forward OnFormPost(index, response[], status, error);
public  OnFormPost(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] form %d failed: error=%d", index, error);
        return 1;
    }

    printf("[example] form %d ok: status=%d", index, status);
    return 1;
}
