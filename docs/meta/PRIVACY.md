# Privacy & Permissions

Kistaverk is offline-first. The app does not request Internet permission and processes data locally. Sensitive Android permissions are requested only when needed and for a single purpose:

- **Camera**: Used only for QR scanning (QR generator/receiver). Frames are decoded in memory via JNI and are not stored or uploaded unless the user explicitly saves a decoded payload.
- **Location**: Used only for Sensor Logger when the user enables GPS logging. Coordinates are written to a local CSV file on the device. Nothing is sent off-device.

No background uploads, analytics, or ad SDKs are included. If you see a permission prompt, it is tied to a specific feature you invoked. You can use other tools (hashing, PDF, text viewer, etc.) without granting these permissions.
