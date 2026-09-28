# Taytay operations

## Installation

Create the unprivileged `taytay` account, install the binary as `/usr/local/bin/taytay`, create `/etc/taytay`, `/var/lib/taytay`, and `/var/log/taytay`, and install `packaging/taytay.service`. Configuration and token files must be owned by `root:taytay` with mode `0640`; the spool must be owned by `taytay:taytay` with mode `0700`.

Validate configuration before starting:

```text
taytay /etc/taytay/taytay.toml
systemctl enable --now taytay
systemctl status taytay
```

## Offline and restart behavior

Source publication completes before an artifact enters the upload ledger. The ledger is atomically rewritten and each artifact is checksum-verified before transfer. On restart, pending jobs are recovered from `ledger.jsonl`; the TUS server offset is authoritative.

Do not delete the spool to resolve a stuck upload. Inspect the job state, connectivity, quota, and server response first. Retain the artifact until Lunsaran acknowledges completion.

## Disk pressure

Set `quota_bytes` below the filesystem capacity, leave room for the ledger and operating system, and alert when the spool approaches the quota. When full, Taytay applies backpressure and rejects new publication without deleting queued artifacts. Cleanup is only safe after terminal completion and an explicit retention policy.

## Upgrades and rollback

Stop the service, replace the binary, run configuration validation, and start the service. Keep the previous binary available for rollback. Never replace or remove `/var/lib/taytay` during an upgrade.

