# Compatibility matrix

| Source | CI fixture | Hardware acceptance | Status |
| --- | --- | --- | --- |
| NVR/filesystem export | filesystem adapter tests | SMB/NFS share disconnect test | Implemented |
| V4L2 USB camera | format/reconnect state tests | camera/capture-card matrix | Boundary implemented |
| RTSP | stream-health state tests | codec/clock/reconnect matrix | Boundary implemented |
| ONVIF Profile T/G/M | probe/event fixtures | device discovery and recording matrix | Parser boundary implemented |
| Drone media/MAVLink sidecar | field-file adapter | flight-device acceptance | File boundary implemented |
| LAS/LAZ LiDAR | extension/checksum adapter | sensor/CRS acceptance | File boundary implemented |

Hardware acceptance must record Linux version, device model, firmware, codec/pixel format, network conditions, reconnect result, and artifact checksum. Credentials and customer metadata must not be included.

