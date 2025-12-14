# C Scripting Feature

The C Scripting feature allows users to compile and run C code directly within the application using the MIR lightweight JIT compiler.

## Features

- **JIT & Interpreter Modes:** Toggle between high-performance JIT compilation and a stable Interpreter mode.
- **Benchmarking:** Measure compilation and execution times with automatic repetition.
- **Background Execution:** Run scripts in a background worker thread to keep the UI responsive.
- **Standard Library Support:** Basic support for `printf`, `malloc`, `free`, `strtol`, `atoi`, etc.
- **UUID Integration:** Generate UUIDs using the Rust `uuid` crate via the `uuid_v4_gen()` C function.

## Integration Details

The C runtime is provided by `mir-sys`. We expose a set of standard libc functions and custom Rust bridges to the C environment using `MIR_load_external`.

### Exposed Functions

- `printf`
- `puts`
- `malloc`
- `free`
- `strtol`
- `atoi`
- `fflush`
- `uuid_v4_gen`: Returns a pointer to a C-string containing a new UUID v4. Caller must `free()` it.

## Threading and Output Capture Risk

Capturing `stdout` and `stderr` from the executed C code relies on `libc::dup2` to redirect the file descriptors to a pipe.

**Critical Warning:** `dup2` modifies the file descriptor table for the **entire process**.

- **Concurrency Risk:** If multiple threads attempt to redirect or use `stdout`/`stderr` simultaneously (e.g., running parallel tests or multiple script executions at once), they will interfere with each other, leading to race conditions where output is lost, intermingled, or sent to the wrong destination.
- **Pipe Deadlock Prevention:** To prevent deadlocks where the C code blocks writing to a full pipe and the Rust side blocks waiting for the C code to finish, a dedicated reader thread is spawned to continuously drain the pipe's output during C code execution. This addresses buffer filling issues for a single execution.
- **Testing:** Unit tests that verify output capture **must** run sequentially. Use `cargo test -- --test-threads=1` to avoid failures.
- **Production:** The application ensures that only one C script execution captures output at a time, or accepts that global stdout redirection affects all threads. Background execution runs in a dedicated worker, but global fd changes still apply process-wide.

## Future Improvements

- Isolate execution in a separate process (e.g., using `fork`/`exec` on Android) to safely capture output without global side effects.
- Expand the standard library shim.
