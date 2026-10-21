---
title: Compatibility
description: Taytay source boundaries and acceptance expectations
---

# Compatibility

This reference lists the source boundaries Taytay exposes and the evidence
needed before accepting a particular device or media workflow. A boundary
being implemented does not mean every device, codec, firmware version, or
network condition has passed hardware acceptance.

| Source | Automated evidence | Hardware acceptance | Status |
| --- | --- | --- | --- |
| NVR or filesystem export | Filesystem adapter tests | SMB/NFS share disconnect test | Implemented |
| V4L2 USB camera | Format and reconnect state tests | Camera and capture-card matrix | Boundary implemented |
| RTSP | Stream-health state tests | Codec, clock, and reconnect matrix | Boundary implemented |
| ONVIF Profile T/G/M | Probe and event fixtures | Device discovery and recording matrix | Parser boundary implemented |
| Drone media or MAVLink sidecar | Field-file adapter tests | Flight-device acceptance | File boundary implemented |
| LAS/LAZ LiDAR | Extension and checksum adapter tests | Sensor and CRS acceptance | File boundary implemented |

## Hardware acceptance evidence

Record the Linux version, device model, firmware, codec or pixel format,
network conditions, reconnect result, and artifact checksum. Do not include
credentials, signed URLs, customer metadata, or media content in the evidence.

Use the [source adapters guide](sources.md) for adapter contracts and the
[development guide](development.md) for the required test scenarios.
