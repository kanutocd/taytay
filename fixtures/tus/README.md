# TUS fixtures

The upload traits model the TUS 1.0 creation, `HEAD`, and `PATCH` sequence. Test doubles must validate `Tus-Resumable`, `Upload-Length`, `Upload-Offset`, chunk ordering, and offset recovery. Fixture payloads must contain synthetic bytes only.
