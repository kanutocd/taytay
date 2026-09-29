# Security policy

Taytay handles source credentials and media on Linux edge devices. Never place
tokens, camera passwords, signed URLs, provider credentials, or media payloads
in issues, pull requests, logs, fixtures, or test output.

## Reporting a vulnerability

Please use [GitHub's private vulnerability reporting](https://github.com/kanutocd/taytay/security/advisories/new)
instead of opening a public issue. Include the affected version or commit, a
minimal reproduction, impact, and any safe mitigation. Do not include real
credentials or customer data.

If private reporting is unavailable, open a minimal issue asking for a private
contact without disclosing exploit details.

## Security boundaries

- Run the reference service as an unprivileged account.
- Keep token files inaccessible to group and other users.
- Keep source credentials local to Taytay.
- Treat upload URLs, authorization headers, and source metadata as sensitive.
- Review adapter changes for leakage through filenames, labels, diagnostics, or
  error messages.

See the [security documentation](https://kanutocd.github.io/taytay/security/)
for the consumer-facing trust model and deployment checklist.
