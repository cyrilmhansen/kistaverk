# Algorithmic Audio Synthesizer

## Overview
The Algorithmic Audio Synthesizer is a powerful feature that allows users to generate sound in real-time by writing C code. It leverages the **Mir JIT** (Just-In-Time) compiler to compile the user's C code into native machine code on the fly, providing high-performance audio synthesis suitable for real-time applications.

## Architecture

### 1. Audio Engine (`SynthesizerEngine`)
*   **Backend:** Uses `cpal` (Cross-Platform Audio Library) to communicate with the system's audio device (e.g., AudioTrack on Android, ALSA/PulseAudio on Linux).
*   **Sample Rate:** Automatically adapts to the device's native sample rate (typically 44.1kHz or 48kHz).
*   **Safety:** The engine uses an `AtomicPtr` to hold the address of the current rendering function. This allows for lock-free, glitch-free hot-swapping of the synthesis algorithm while the audio thread is running.

### 2. JIT Compiler (`mir-sys`)
*   **Compiler:** Uses the Mir C JIT compiler (embedded via `mir-sys`).
*   **Optimization:** Configured with optimization level 1 for a balance between compilation speed and execution performance.
*   **Context Management:** Maintains a persistent `MIR_context` that is refreshed whenever the user recompiles the code.

## Usage

### The `render` Function
Users define a single function with the following signature:

```c
double render(double t, double p1, double p2)
```

*   `t`: The current time in seconds (since the synthesizer started).
*   `p1`: User parameter 1 (controlled by the UI input).
*   `p2`: User parameter 2 (controlled by the UI input).
*   **Returns:** A `double` representing the amplitude of the audio signal (typically between -1.0 and 1.0).

### Built-in Math Functions
The JIT environment includes standard math library functions linked statically:
*   `sin`, `cos`, `tan`
*   `exp`, `log`
*   `pow`
*   `fmod`

### Examples

#### Basic Sine Wave
```c
double render(double t, double p1, double p2) {
    // p1: Frequency (default 440)
    // p2: Amplitude (0.0 to 1.0)
    double freq = p1 > 0 ? p1 : 440.0;
    double amp = p2 > 0 ? p2 : 0.5;
    return sin(t * freq * 6.28318) * amp;
}
```

#### FM Synthesis
```c
double render(double t, double p1, double p2) {
    // p1: Carrier Frequency
    // p2: Modulation Index
    double freq = p1 > 0 ? p1 : 220.0;
    double mod_index = p2 > 0 ? p2 : 5.0;
    
    double modulator = sin(t * freq * 0.5 * 6.28);
    return sin(t * freq * 6.28 + modulator * mod_index) * 0.5;
}
```

## Technical Implementation Details
*   **Thread Safety:** The audio callback runs on a high-priority system thread. It reads the function pointer from an `AtomicPtr`. The main UI thread compiles the new code, gets the new function address, and atomically swaps the pointer.
*   **State Management:** `SynthesizerState` in `state.rs` persists the source code and parameter values across screen navigations.
*   **Error Handling:** Compilation errors are captured and displayed in the UI without interrupting the audio stream (the previous valid function continues to play).
