# MIR API Reference

This document provides a comprehensive reference for the MIR (Medium Internal Representation) scripting API in kistaverk.

## 📚 API Overview

The MIR API provides functionality for:

- **MIR Code Execution**: Run MIR programs in both JIT and interpreter modes
- **State Management**: Manage MIR scripting state
- **Error Handling**: Comprehensive error reporting
- **Performance Measurement**: Execution time tracking

## 🧩 Core Data Structures

### MirScriptingState

```rust
pub struct MirScriptingState {
    /// MIR source code
    pub source: String,
    
    /// Entry function name
    pub entry: String,
    
    /// Execution output
    pub output: String,
    
    /// Error message (if any)
    pub error: Option<String>,
}
```

**Methods**:

| Method | Description | Returns |
|--------|-------------|---------|
| `new()` | Create new MIR state | `MirScriptingState` |
| `execute_jit()` | Execute with JIT compilation | `Option<u128>` |
| `execute_interp()` | Execute with interpretation | `Option<u128>` |
| `clear_output()` | Clear output and errors | `()` |
| `clear_source()` | Clear source code | `()` |

### MirExecMode

```rust
enum MirExecMode {
    Jit,    // Just-In-Time compilation
    Interp, // Interpretation
}
```

## 🔧 API Functions

### State Management

#### `MirScriptingState::new()`

**Description**: Create a new MIR scripting state

**Returns**: `MirScriptingState` - New state instance

**Example**:
```rust
let mut state = MirScriptingState::new();
```

#### `MirScriptingState::clear_output()`

**Description**: Clear output and error messages

**Example**:
```rust
state.clear_output();
```

#### `MirScriptingState::clear_source()`

**Description**: Clear source code and output

**Example**:
```rust
state.clear_source();
```

### Execution Functions

#### `MirScriptingState::execute_jit()`

**Description**: Execute MIR code using JIT compilation

**Returns**: `Option<u128>` - Execution time in milliseconds

**Example**:
```rust
state.source = "m_calc: module...".to_string();
state.entry = "main".to_string();
let runtime = state.execute_jit();
println!("JIT runtime: {:?}ms", runtime);
```

#### `MirScriptingState::execute_interp()`

**Description**: Execute MIR code using interpretation

**Returns**: `Option<u128>` - Execution time in milliseconds

**Example**:
```rust
state.source = "m_calc: module...".to_string();
state.entry = "main".to_string();
let runtime = state.execute_interp();
println!("Interpreter runtime: {:?}ms", runtime);
```

### Utility Functions

#### `render_mir_scripting_screen()`

**Description**: Render MIR scripting UI

**Parameters**:
- `state: &AppState` - Application state

**Returns**: `serde_json::Value` - UI JSON representation

**Example**:
```rust
let ui_json = render_mir_scripting_screen(&app_state);
```

#### `handle_mir_scripting_actions()`

**Description**: Handle MIR scripting actions

**Parameters**:
- `state: &mut AppState` - Mutable application state
- `action: Action` - Action to handle

**Returns**: `Option<serde_json::Value>` - Optional UI update

**Example**:
```rust
if let Some(ui_update) = handle_mir_scripting_actions(&mut app_state, action) {
    send_ui_update(ui_update);
}
```

## 📝 MIR Language Reference

### Module Syntax

```mir
module_name: module
             export function1, function2
             import external_function
             
function1:  func return_type, param1:type1, param2:type2
            local var1:type1, var2:type2
            [instructions]
            ret return_value
            endfunc
            
            endmodule
```

### Data Types

| Type | Description | Size |
|------|-------------|------|
| `i8` | 8-bit integer | 1 byte |
| `i16` | 16-bit integer | 2 bytes |
| `i32` | 32-bit integer | 4 bytes |
| `i64` | 64-bit integer | 8 bytes |
| `u8` | 8-bit unsigned | 1 byte |
| `u16` | 16-bit unsigned | 2 bytes |
| `u32` | 32-bit unsigned | 4 bytes |
| `u64` | 64-bit unsigned | 8 bytes |
| `f32` | 32-bit float | 4 bytes |
| `f64` | 64-bit float | 8 bytes |
| `p` | Pointer | 8 bytes |

### Instructions

#### Arithmetic

| Instruction | Description | Example |
|-------------|-------------|---------|
| `add` | Addition | `add r, a, b` |
| `sub` | Subtraction | `sub r, a, b` |
| `mul` | Multiplication | `mul r, a, b` |
| `div` | Division | `div r, a, b` |
| `mod` | Modulo | `mod r, a, b` |
| `neg` | Negation | `neg r, a` |
| `abs` | Absolute value | `abs r, a` |

#### Control Flow

| Instruction | Description | Example |
|-------------|-------------|---------|
| `jmp` | Unconditional jump | `jmp label` |
| `beq` | Branch if equal | `beq label, a, b` |
| `bne` | Branch if not equal | `bne label, a, b` |
| `bgt` | Branch if greater | `bgt label, a, b` |
| `blt` | Branch if less | `blt label, a, b` |
| `bge` | Branch if greater/equal | `bge label, a, b` |
| `ble` | Branch if less/equal | `ble label, a, b` |

#### Memory

| Instruction | Description | Example |
|-------------|-------------|---------|
| `mov` | Move value | `mov r, 42` |
| `alloca` | Allocate memory | `alloca array, 100` |
| `load` | Load from memory | `mov r, i64:(array, index)` |
| `store` | Store to memory | `mov i64:(array, index), r` |

#### Function Calls

| Instruction | Description | Example |
|-------------|-------------|---------|
| `call` | Call function | `call func, r, arg1, arg2` |
| `ret` | Return from function | `ret r` |

## 🛡️ Error Handling

### Error Types

| Error Type | Description | Example |
|------------|-------------|---------|
| `SyntaxError` | Invalid MIR syntax | Missing `endmodule` |
| `LinkError` | Linking failed | Missing function |
| `RuntimeError` | Execution error | Division by zero |
| `MemoryError` | Memory access error | Invalid pointer |
| `TimeoutError` | Execution timeout | Infinite loop |

### Error Reporting

```rust
match state.error {
    Some(ref error) => {
        println!("MIR Error: {}", error);
        // Handle specific error types
        if error.contains("syntax") {
            show_syntax_error_ui();
        } else if error.contains("link") {
            show_link_error_ui();
        }
    }
    None => {
        println!("MIR Execution successful");
    }
}
```

## 📊 Performance API

### Execution Time Measurement

```rust
let start_time = Instant::now();
let runtime = state.execute_jit();
let end_time = Instant::now();

println!("Total time: {:?}", end_time - start_time);
println!("MIR runtime: {:?}ms", runtime);
```

### Benchmarking

```rust
fn benchmark_mir_code(source: &str, entry: &str, iterations: u32) -> BenchmarkResult {
    let mut state = MirScriptingState::new();
    state.source = source.to_string();
    state.entry = entry.to_string();
    
    // Warmup
    for _ in 0..5 {
        state.execute_jit();
    }
    
    // Benchmark
    let mut times = Vec::new();
    for _ in 0..iterations {
        state.clear_output();
        if let Some(runtime) = state.execute_jit() {
            times.push(runtime);
        }
    }
    
    BenchmarkResult::from_times(times)
}
```

## 🔄 Integration API

### Math Tool Integration

```rust
// Convert math expression to MIR
fn math_to_mir(expr: &str) -> Result<String, String> {
    // Parse expression
    let ast = parse_expression(expr)?;
    
    // Generate MIR code
    let mir_code = generate_mir(&ast)?;
    
    Ok(mir_code)
}

// Execute math expression via MIR
fn execute_via_mir(expr: &str) -> Result<Number, String> {
    let mir_code = math_to_mir(expr)?;
    
    let mut state = MirScriptingState::new();
    state.source = mir_code;
    state.entry = "main".to_string();
    
    if let Some(runtime) = state.execute_jit() {
        // Parse result from output
        parse_mir_result(&state.output)
    } else {
        Err(state.error.unwrap_or("Unknown error".to_string()))
    }
}
```

### Function Library Integration

```rust
struct MirFunctionLibrary {
    functions: HashMap<String, MirFunction>,
}

struct MirFunction {
    name: String,
    source: String,
    entry: String,
    description: String,
    parameters: Vec<MirParameter>,
    return_type: String,
}

impl MirFunctionLibrary {
    fn add_function(&mut self, name: String, source: String, entry: String) {
        self.functions.insert(name.clone(), MirFunction {
            name,
            source,
            entry,
            description: String::new(),
            parameters: Vec::new(),
            return_type: "i64".to_string(),
        });
    }
    
    fn execute(&self, name: &str, args: Vec<Number>) -> Result<Number, String> {
        if let Some(func) = self.functions.get(name) {
            let mut state = MirScriptingState::new();
            state.source = func.source.clone();
            state.entry = func.entry.clone();
            
            // Set up arguments (implementation depends on MIR ABI)
            // ...
            
            if let Some(runtime) = state.execute_jit() {
                parse_mir_result(&state.output)
            } else {
                Err(state.error.unwrap_or("Unknown error".to_string()))
            }
        } else {
            Err("Function not found".to_string())
        }
    }
}
```

## 📁 File Structure

```
rust/src/
├── features/
│   └── mir_scripting.rs       # Main MIR scripting implementation
├── state.rs                  # App state with MIR state
└── router.rs                 # Action routing for MIR
```

## 📚 Related Documents

- **[MIR Scripting Overview](../features/mir-scripting/overview.md)** - MIR scripting features
- **[MIR Scripting Examples](../features/mir-scripting/examples.md)** - Practical MIR examples
- **[MIR Integration](../features/mir-scripting/integration.md)** - Integration with other features
- **[System Architecture](../architecture/overview.md)** - Overall system architecture

**Last updated:** 2025-12-14