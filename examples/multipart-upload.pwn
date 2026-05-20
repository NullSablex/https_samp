// multipart-upload.pwn — file upload via multipart/form-data.
//
// Demonstrates:
//   * Building a multipart payload mixing text fields and a file field.
//   * Letting the plugin set Content-Type with the correct boundary.

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    https_multipart_add_text("title", "weekly report");
    https_multipart_add_text("author", "erick");
    https_multipart_add_file("attachment", "report.txt", "scriptfiles/report.txt");

    https(1, HTTPS_POST, "https://httpbin.org/post", "", "OnUpload");
    return 1;
}

forward OnUpload(index, response[], status, error);
public  OnUpload(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] upload failed: error=%d", error);
        return 1;
    }
    printf("[example] upload ok: status=%d", status);
    return 1;
}
