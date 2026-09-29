---
title: Source adapters
description: Build source-independent edge integrations
---

# Source adapters

Taytay keeps acquisition behind typed seams. You can start with a completed
file and later replace it with a device adapter without changing the spool or
uploader.

## Adapter contract

An adapter should:

- identify itself with an opaque `SourceId`;
- emit only complete, stable artifacts;
- preserve capture time and source metadata when available;
- avoid logging credentials or sensitive filenames; and
- handle source disconnects without deleting already-published jobs.

The core includes policy primitives for these source families:

| Source | Current public boundary | Integration work outside the crate |
| --- | --- | --- |
| Filesystem/NVR | Stability checks and completed-file publication | SMB/NFS watcher and vendor export behavior |
| V4L2 | Format negotiation, reconnect state, bounded segmenting | Device I/O, encoder, preview, and capture loop |
| RTSP | Timestamp/codec validation, bounded segment buffer, reconnect state | Supervised GStreamer/FFmpeg process and camera matrix |
| ONVIF | Discovery/profile/event parsing seams | Network discovery, Profile T/G device behavior, credentials |
| Drone/LiDAR | Validated sidecar metadata and file association | MAVLink reader, LAS/LAZ parser, flight/sensor acceptance |

## Filesystem/NVR pattern

For exported media, wait for an atomic producer rename, a producer marker, or
two consecutive stable observations before publishing. Do not upload a file
whose size is still changing.

## Device protocol pattern

Keep protocol-specific work in the adapter. The adapter may supervise a
capture process, negotiate a format, or translate an event into a capture
window, but it should hand the common spool a normal `Artifact`. The queue
should not know whether the bytes came from ONVIF or a local directory.

## Metadata

Metadata is part of the artifact contract and should be treated as sensitive by
default. Prefer stable IDs, timestamps, codec or sensor facts, and bounded
structured values. Never put passwords, bearer tokens, signed URLs, or raw
media payloads into metadata or logs.
