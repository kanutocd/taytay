---
title: Architecture
description: Taytay ownership boundaries and upload lifecycle
---

# Architecture

Taytay separates source acquisition, local durability, and remote transfer.
This lets an integrator replace a camera or transport without changing the
durability contract.

## Ownership boundaries

```mermaid
flowchart TB
    subgraph edge[Linux edge device]
        source[Source adapter]
        spool[Spool and ledger]
        policy[Retry, quota, checksum, credential policy]
        transfer[ArtifactUploader]
        source --> spool --> policy --> transfer
    end
    lunsaran[Lunsaran<br/>identity, org/project scope,<br/>upload-session policy]
    brutus[Brutus<br/>TUS offsets, bytes,<br/>provider execution]
    transfer --> lunsaran
    transfer --> brutus
```

Taytay owns source credentials and local state. It must never receive or
persist customer storage-provider credentials. Lunsaran owns the authorization
decision for a new upload session. Brutus owns the byte-level TUS behavior.

## Upload lifecycle

```mermaid
sequenceDiagram
    participant S as Source adapter
    participant P as Taytay spool
    participant L as Taytay ledger
    participant C as Lunsaran client
    participant B as Brutus/TUS

    S->>P: publish completed bytes
    P->>L: insert Published job + checksum
    L-->>P: durable job
    P->>C: create scoped upload session
    C->>B: create or resume TUS upload
    loop until complete
        C->>B: HEAD / PATCH from committed offset
        B-->>C: authoritative offset
        C-->>L: progress and retry state
    end
    C-->>L: Completed receipt
    P->>P: retain until cleanup policy permits deletion
```

## Consumer responsibilities

Your application is responsible for:

- Detecting when source output is complete before publishing it.
- Assigning stable artifact and source identifiers.
- Keeping source credentials local and redacting source metadata in logs.
- Supplying a persistent spool path with enough quota for offline operation.
- Running the upload worker and handling retryable versus terminal errors.
- Provisioning Lunsaran scope and rotating device credentials.

Taytay does not make an incomplete file complete, infer tenant authorization,
or provide a hosted administration UI.
