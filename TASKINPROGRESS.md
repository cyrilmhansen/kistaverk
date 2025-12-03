# Task In Progress: None

## Completed Task: QR Data Transfer (Sender)
*   **Status:** Implemented and Verified.
*   **Date:** 2025-12-03
*   **Summary:** Implemented end-to-end QR data transfer sender.
    *   **Rust Core:** `QrSlideshowState` handles file loading, chunking (512 bytes), and QR generation (`QRTX|i/n|base64`).
    *   **UI:** Added `QrSlideshow` screen with file picker, play/pause, speed control, and next/prev frame navigation.
    *   **Android:** `MainActivity.kt` handles `auto_refresh_ms` to drive the slideshow loop.
    *   **Tests:** Unit tests for chunking and frame advancement logic verified in `rust/src/features/qr_transfer.rs`.
