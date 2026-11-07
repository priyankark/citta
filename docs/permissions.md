# Device permissions from any device

Open **Permissions** in the control panel and select a device. The selected device stores the policy in its own SQLite database and enforces it on execution. You can use any updated, automatically paired device's panel to edit another device. No pairing codes or manual management-token exchange are needed.

The three execution modes are:

- **Restricted:** only the selected installed capabilities may run unattended.
- **Allow all:** every installed capability may run unattended, including capabilities added later. The allowed-origin selection still applies.
- **Pause execution:** stop affected active Citta jobs and claim no new work.

Choose **All trusted devices** to include newly paired devices automatically, or select specific devices allowed to send work. Automatic peers use stable Tailscale device identities; reinstalling a device with a new identity does not inherit an old grant. Local work and explicitly configured legacy peers have separate identity keys. Removing a device from Tailscale trust revokes automatic access independently of policy.

These settings govern Citta capabilities, not arbitrary programs or existing externally managed conversations. They cannot install a capability, change its executable, modify its sandbox, widen a harness approval policy, or remove configured time/output/concurrency limits. Allow-all is unattended access to registered capabilities, not a sandbox escape switch. MXC isolation and per-task approval holds are not implemented.

## Enforcement and changes

Workers check current policy before claiming work and atomically recheck it when launching the process. Policy changes cancel affected running jobs, including when permissions are quickly re-enabled. Such jobs become interrupted and are never automatically replayed. Revocation cannot undo effects that already occurred.

Denied capabilities/origin channels are not claimed. Queued tasks remain eligible until their queue deadline and can run if permission is later granted; expired queues become interrupted. A request submitted remotely into the executing device's own queue retains the authenticated requester's identity so it cannot masquerade as local work. For a remote origin queue, the authenticated origin device is the delegation principal. A locally claimed request denied by its requester scope receives a failed outcome without launching a process.

Each save includes the revision originally loaded by the editor. Concurrent edits return a conflict; reload the saved policy before editing again. An unreachable device is never shown as successfully updated, and uncertain writes are not retried automatically. Reload after a connection error to learn whether the target committed the change.

Policies and their revision/actor/timestamp history commit in one SQLite transaction. The panel shows the latest twenty changes; older records remain in the database. Initial migration preserves execution of the currently installed capabilities in Restricted mode, for all currently trusted origins. New capabilities subsequently require an explicit grant unless Allow all is selected. No existing device is silently changed to Allow all.

## Owner access and automatic management pairing

The daemon has separate credentials for task traffic and owner management. The local CLI reads `data/control.token` when issuing a panel login link; MCP and worker task clients use the task credential. Task credentials cannot read/write the management API or mint browser sessions. Management credentials cannot authenticate task API calls.

Same-owner daemons establish a second verified HTTPS challenge channel under `/v1/control/enroll/*`. It uses independent challenges, credentials, and `data/control-enrollment.json`, while retaining the existing owner/netmap checks, TLS verification, expiry, and revocation behavior. These management credentials remain in the daemons; they are never returned to browser JavaScript or delegated workers. They authorize the owner's paired daemons to manage each other, even when work from one of those devices is blocked. Work permission is separate from owner administration.

A copied legacy node task token is indistinguishable from local task authority. Use automatic identity-bound peer credentials for origin restrictions; remove old shared-token grants when migrating. Explicit legacy peer rules identify the configured pull channel, not the holder of a copied local token.

This is a single-owner trust model. A process with unrestricted access to the service user's private files can read management credentials; Citta does not claim isolation from that OS user or a compromised owner device. Use real OS/harness isolation for delegated code.

## Open the panel on another device

```sh
citta --config "$HOME/.config/citta/config.toml" panel --network
```

This returns a short-lived login link on the daemon's private Tailscale HTTPS address. Open it in a browser on a device that can reach that address. Local access without `--network` continues to work. Login links are single-use and expire after two minutes; owner browser sessions last eight hours and end on logout or daemon restart. HTTPS sessions use Secure, HttpOnly, SameSite cookies. Do not share a login link with someone who should not manage your mesh. No public Funnel or cloud-hosted control plane is enabled.

The backend accepts only its configured loopback host or the current verified Tailscale HTTPS origin, rejects cross-site browser requests, and requires a panel-specific header on policy writes. Device selection resolves only enrolled management clients, never arbitrary caller-supplied URLs.

## Upgrade requirements

Both the panel-hosting daemon and the selected remote daemon need the permissions-enabled build. Older devices can continue ordinary task traffic but cannot be remotely configured until upgraded. The panel reports that management is unavailable rather than suggesting a change was applied.

Storage advances to version 3 so older binaries cannot bypass saved policies. Back up the stopped database, config, binary, and private credential files before upgrading. Never force a downgrade by editing `user_version`. See [operations](operations.md) for rollback and reconciliation.
