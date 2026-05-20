// mtls.pwn — install a client certificate identity for mutual TLS.
//
// Demonstrates:
//   * Loading a combined PEM (cert + key) from disk.
//   * Issuing a request that automatically uses the installed identity.
//   * Clearing the identity when done.

#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    // The PEM file must contain both the client certificate and the private
    // key. Files larger than 256 KiB are refused by the loader.
    if (!https_mtls_set_pem_file("scriptfiles/client-identity.pem"))
    {
        print("[example] failed to install mTLS identity");
        return 1;
    }

    // All subsequent requests use the installed identity until cleared.
    https(1, HTTPS_GET, "https://mtls.example.com/whoami", "", "OnMtlsDone");
    return 1;
}

forward OnMtlsDone(index, response[], status, error);
public  OnMtlsDone(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] mtls req %d failed: error=%d", index, error);
    }
    else
    {
        printf("[example] mtls req %d ok: status=%d", index, status);
    }

    // Remove the identity now that the protected call is done.
    https_mtls_clear();
    return 1;
}
