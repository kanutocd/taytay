# Taytay engineering instructions

## Mission

Build a dependable Linux edge bridge that turns source media and metadata into durable, resumable Lunsaran uploads.

## Boundaries

- Taytay owns source adapters, local buffering, upload scheduling, retries, checksums, local state, and device-side observability.
- Lunsaran owns users, organizations, projects, authorization, routing, workflows, and upload-session policy.
- Brutus owns TUS protocol behavior, upload offsets, upload bytes, and provider execution.
- Taytay never receives or persists customer storage-provider credentials.
- Source credentials remain on the edge and must never be logged or sent to Lunsaran.

## Engineering rules

- Use stable Rust and target supported Linux distributions explicitly.
- Keep source adapters behind small typed traits; do not couple the queue or uploader to CCTV, ONVIF, RTSP, or a vendor SDK.
- Treat every artifact as a durable job until the server confirms completion.
- Make retries idempotent and safe across process restarts.
- Use atomic file publication, checksums, bounded concurrency, and backpressure.
- Do not lose queued artifacts when the network or process stops.
- Keep handlers and adapters thin; put policy in explicit modules.
- Never log credentials, tokens, signed URLs, media payloads, or sensitive source metadata.
- Prefer established protocols: ONVIF, RTSP/RTP, V4L2, filesystem APIs, HTTPS, and TUS.
- Add API or event contract changes to `OPENAPI.md` and an ADR before implementation.
- The private architecture, implementation, API-contract, quality, and
  `lunsaran-entregar` integration harness is maintained in the companion
  `taytay-saas/docs/taytay-linux-edge-bridge` repository; keep public crate
  documentation and source contracts self-contained.

## Required checks

Before merging a change, run the narrowest relevant tests, `cargo fmt --all -- --check`, `cargo check --workspace`, relevant integration tests, and clippy with warnings denied. Run `scripts/quality.sh` for the complete gate and `scripts/coverage.sh` for coverage. Follow the staged thresholds and exclusions in `QUALITY.md`. Report commands that were actually run.

## Git safety

Do not reset, clean, rebase, force-push, or discard user changes. Keep commits coherent and update `CHANGELOG.md` for user-visible or architectural changes.
