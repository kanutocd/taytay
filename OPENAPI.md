# Taytay API and contract harness

## Contract boundary

Taytay is initially an outbound Linux edge client. Its primary external contracts are:

1. Lunsaran's authenticated REST control-plane API for organization/project context, upload-session creation, and lifecycle reporting.
2. Brutus's TUS 1.0 endpoint for resumable byte transfer.
3. Source protocols such as ONVIF, RTSP/RTP, V4L2, filesystem APIs, and vendor APIs.

Taytay must not invent a second upload protocol or call object-storage providers directly.

## REST/OpenAPI rules for any future local API

- Model resources as nouns and use HTTP methods for actions.
- Use stable plural paths and explicit versioning.
- Keep organization and project context explicit and server-authorized.
- Use idempotency keys for create/retry operations.
- Return typed error bodies with stable codes.
- Define pagination, time formats, size limits, and retry behavior.
- Never expose camera credentials, provider credentials, access tokens, signed URLs, or raw media in responses.
- Document every endpoint in an OpenAPI document before implementation.

## Planned local resources

A future local administration API may expose read-only or operator-scoped resources such as:

- `GET /v1/status`
- `GET /v1/sources`
- `GET /v1/jobs`
- `POST /v1/jobs/{jobId}/retry`
- `POST /v1/sources/{sourceId}/pause`
- `POST /v1/sources/{sourceId}/resume`

These are not implemented or committed as a public contract yet. Local access must be authenticated and bound to the device operator.

## Lunsaran integration requirements

The generated client must be pinned to a reviewed Lunsaran OpenAPI version. Upload-session responses must provide a scoped Brutus URL, expiry, organization/project binding, and protocol capabilities without exposing provider credentials.

## TUS requirements

Implement the required TUS creation, `HEAD` offset recovery, `PATCH` chunk transfer, checksum where negotiated, termination behavior, and server capability discovery. Preserve the server offset as the source of truth after every retry.
