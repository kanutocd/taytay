# Taytay

Taytay is a Rust Linux edge bridge for Lunsaran. It receives artifacts from local or networked sources, stores them durably while offline, and uploads them to Lunsaran through Brutus using TUS.

The first source adapter targets existing CCTV/NVR installations. The core remains source-independent so later adapters can support drones, LiDAR systems, USB cameras, filesystems, and other field devices.

## Project status

The `0.0.1` harness and durable local/upload foundation are implemented, including operator diagnostics, secure resume state, bounded source segmenting, event deduplication, and health counters. Source-specific hardware integrations, live service clients, and the `entregar` adapter remain behind typed boundaries and require external libraries, devices, or acceptance fixtures before production release.

The service validates TOML configuration, publishes artifacts atomically into a quota-bounded spool, persists resumable jobs, verifies SHA-256 checksums, and exposes source/control-plane/TUS seams for local fixtures. See [operations](docs/OPERATIONS.md), [security](docs/SECURITY.md), and [compatibility](docs/COMPATIBILITY.md).

## Name

*Taytay* is a Hiligaynon word for bridge. The name describes the edge-to-platform role rather than a specific source device.
