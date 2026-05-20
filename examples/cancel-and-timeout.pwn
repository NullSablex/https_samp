// cancel-and-timeout.pwn — per-request timeout and cancellation.
//
// Demonstrates:
//   * https_set_timeout_once to give a single call more (or less) time.
//   * https_cancel to drop the callback for a request the gamemode no longer needs.

#include <a_samp>
#include <https_samp>

new g_playerRequestIndex[MAX_PLAYERS];

public OnPlayerConnect(playerid)
{
    // Give this request 3 seconds total instead of the default 12.
    https_set_timeout_once(3000);
    https_set_bearer_once("demo-token");

    g_playerRequestIndex[playerid] = playerid + 1000;
    https(g_playerRequestIndex[playerid], HTTPS_GET,
          "https://httpbin.org/delay/1", "", "OnPlayerLookup");
    return 1;
}

public OnPlayerDisconnect(playerid, reason)
{
    // Drop the pending callback if the player left before it arrived.
    https_cancel(g_playerRequestIndex[playerid]);
    return 1;
}

forward OnPlayerLookup(index, response[], status, error);
public  OnPlayerLookup(index, response[], status, error)
{
    new playerid = index - 1000;
    if (!IsPlayerConnected(playerid))
    {
        // Defensive guard: cancel may race with delivery in unusual cases.
        return 1;
    }
    if (error == HTTPS_ERROR_TIMEOUT)
    {
        printf("[example] lookup for %d timed out", playerid);
        return 1;
    }
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[example] lookup for %d failed: %d", playerid, error);
        return 1;
    }
    printf("[example] lookup for %d ok: status=%d", playerid, status);
    return 1;
}
