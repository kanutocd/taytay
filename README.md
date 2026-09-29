# Taytay

Taytay is a Rust Linux edge bridge for Lunsaran. It receives artifacts from local or networked sources, stores them durably while offline, and uploads them to Lunsaran through Brutus using TUS.

Consumer documentation is published with Zensical at
<https://kanutocd.github.io/taytay>. Start with the [quick start](docs/quickstart.md)
or the [Rust crate guide](docs/crate-guide.md) if you are embedding Taytay in
an OEM device, edge worker, or source integration.

The first source adapter targets existing CCTV/NVR installations. The core remains source-independent so later adapters can support drones, LiDAR systems, USB cameras, filesystems, and other field devices.

## Project status

The durable local/upload foundation and the published `lunsaran-entregar`
integration are implemented, including operator diagnostics, secure resume
state, bounded source segmenting, event deduplication, health counters, and
deterministic mock acceptance. Source-specific hardware capture, live service
acceptance, and field compatibility still require external devices or
environments.

The service validates TOML configuration, publishes artifacts atomically into a quota-bounded spool, persists resumable jobs, verifies SHA-256 checksums, and exposes source/control-plane/TUS seams for local fixtures. See [operations](docs/OPERATIONS.md), [security](docs/SECURITY.md), and [compatibility](docs/COMPATIBILITY.md).

Private architecture, implementation planning, API-contract, quality, and `lunsaran-entregar` integration harness documents are maintained in the companion `taytay-saas/docs/taytay-linux-edge-bridge` repository. They are not required to build or consume the public crate.

## Name

*Taytay* is a Hiligaynon word for bridge. The name describes the edge-to-platform role rather than a specific source device.

## Integrator strategy

Taytay is a first-party reference integrator of Lunsaran, using the public
`lunsaran-entregar` Rust client library. The same boundary is intended for
third-party integrators and OEMs. A deployment still requires a Lunsaran
account, organization/project authorization, and a scoped device/workload
credential; public source distribution does not bypass Lunsaran identity.

The reusable core is planned for an Apache-2.0 public release after its
credential, spool, and uploader contracts stabilize. Hosted fleet management,
billing, proprietary adapters, and advanced orchestration may remain in a
private SaaS product built on the public core.

The future SaaS product will live in a separate private repository. This
repository remains focused on the public edge workspace, its reference service
and CLI, source adapters, and stable integration contracts. The SaaS product
will consume published Taytay crates instead of coupling its release cycle to
this workspace.
