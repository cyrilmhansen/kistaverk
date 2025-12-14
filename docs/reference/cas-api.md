# CAS API Reference

This document provides a comprehensive reference for the Computer Algebra System (CAS) API in kistaverk.

## 📚 API Overview

The CAS API provides functionality for:

- **Mathematical Computation**: Numerical and symbolic mathematics
- **Precision Control**: Fast (f64) and arbitrary precision (rug::Float) modes
- **Expression Evaluation**: Parse and evaluate mathematical expressions
- **Symbolic Mathematics**: Differentiation and integration
- **Error Handling**: Comprehensive error reporting and estimation

## 🧩 Core Data Structures

### Number

```rust
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Number {
    /// Fast floating-point representation using f64
    Fast(f64),
    /// Arbitrary precision representation using rug::Float
    #[cfg(feature = "precision")]
    Precise(rug::Float),
}
```

**Methods**:

| Method | Description | Returns |
|--------|-------------|---------|
| `from_f64(value: f64)` | Create from f64 | `Number` |
| `to_f64(self)` | Convert to f64 | `f64` |
| `to_precise(self)` | Convert to precise | `Number` |
| `to_fast(self)` | Convert to fast | `Number` |

### MathToolState

```rust
pub struct MathToolState {
    /// Current mathematical expression
    pub expression: String,
    
    /// Expression history
    pub history: Vec<MathHistoryEntry>,
    
    /// Current precision setting (0 = fast, >0 = precise)
    pub precision_bits: u32,
    
    /// Accumulated floating-point error
    pub cumulative_error: f64,
    
    /// Last error (if any)
    pub error: Option<String>,
}
```

**Methods**:

| Method | Description | Returns |
|--------|-------------|---------|
| `clear_history()` | Clear expression history | `()` |
| `toggle_precision()` | Toggle precision mode | `()` |
| `add_to_history(expr: String, result: Number)` | Add to history | `()` |

### MathHistoryEntry

```rust
pub struct MathHistoryEntry {
    /// Expression that was evaluated
    pub expression: String,
    
    /// Result of evaluation
    pub result: String,
    
    /// Estimated error (if available)
    pub error_estimate: Option<f64>,
    
    /// Precision used for calculation
    pub precision_bits: u32,
}
```

## 🔧 API Functions

### Expression Evaluation

#### `evaluate_expression(expr: &str, precision_bits: u32)`

**Description**: Evaluate a mathematical expression

**Parameters**:
- `expr`: Mathematical expression string
- `precision_bits`: Precision setting (0 = fast, >0 = precise)

**Returns**: `Result<Number, String>` - Result or error

**Example**:
```rust
let result = evaluate_expression("sin(pi/2) + 3^2", 0)?;
println!("Result: {}", result.to_f64());
```

#### `tokenize(expr: &str)`

**Description**: Tokenize a mathematical expression

**Parameters**:
- `expr`: Mathematical expression string

**Returns**: `Result<Vec<Token>, String>` - Tokens or error

**Example**:
```rust
let tokens = tokenize("2 + 3 * 4")?;
println!("Tokens: {:?}", tokens);
```

#### `shunting_yard(tokens: Vec<Token>)`

**Description**: Convert tokens to Reverse Polish Notation (RPN)

**Parameters**:
- `tokens`: Token vector

**Returns**: `Result<Vec<Token>, String>` - RPN tokens or error

**Example**:
```rust
let rpn = shunting_yard(tokens)?;
println!("RPN: {:?}", rpn);
```

#### `eval_rpn(rpn: Vec<Token>, precision_bits: u32)`

**Description**: Evaluate RPN expression

**Parameters**:
- `rpn`: RPN token vector
- `precision_bits`: Precision setting

**Returns**: `Result<Number, String>` - Result or error

**Example**:
```rust
let result = eval_rpn(rpn, 0)?;
println!("Result: {}", result.to_f64());
```

### Symbolic Mathematics

#### `differentiate(expr: &str, var: &str)`

**Description**: Symbolic differentiation

**Parameters**:
- `expr`: Mathematical expression
- `var`: Variable to differentiate by

**Returns**: `Result<String, String>` - Derivative expression or error

**Example**:
```rust
let derivative = differentiate("x^2 + sin(x)", "x")?;
println!("Derivative: {}", derivative);
```

#### `integrate(expr: &str, var: &str)`

**Description**: Symbolic integration

**Parameters**:
- `expr`: Mathematical expression
- `var`: Variable to integrate by

**Returns**: `Result<String, String>` - Integral expression or error

**Example**:
```rust
let integral = integrate("2*x", "x")?;
println!("Integral: {}", integral);
```

### Precision Management

#### `set_precision_bits(state: &mut MathToolState, bits: u32)`

**Description**: Set precision level

**Parameters**:
- `state`: Math tool state
- `bits`: Precision bits (0 = fast, 64/128/256 = precise)

**Example**:
```rust
set_precision_bits(&mut state, 128);
```

#### `toggle_precision(state: &mut MathToolState)`

**Description**: Toggle between fast and precise modes

**Parameters**:
- `state`: Math tool state

**Example**:
```rust
toggle_precision(&mut state);
```

### Error Handling

#### `estimate_error(op: Op, a: &Number, b: &Number)`

**Description**: Estimate floating-point error

**Parameters**:
- `op`: Operation performed
- `a`: First operand
- `b`: Second operand

**Returns**: `f64` - Estimated error

**Example**:
```rust
let error = estimate_error(Op::Add, &a, &b);
state.cumulative_error += error;
```

#### `check_overflow(result: &Number)`

**Description**: Check for overflow/underflow

**Parameters**:
- `result`: Result to check

**Returns**: `bool` - True if overflow/underflow detected

**Example**:
```rust
if check_overflow(&result) {
    return Err("Arithmetic overflow".to_string());
}
```

## 📝 Expression Syntax

### Supported Operations

| Operation | Symbol | Example |
|-----------|--------|---------|
| Addition | `+` | `2 + 3` |
| Subtraction | `-` | `5 - 2` |
| Multiplication | `*` | `3 * 4` |
| Division | `/` | `10 / 2` |
| Exponentiation | `^` | `2 ^ 3` |
| Parentheses | `()` | `(2 + 3) * 4` |

### Functions

| Function | Description | Example |
|----------|-------------|---------|
| `sin(x)` | Sine | `sin(pi/2)` |
| `cos(x)` | Cosine | `cos(0)` |
| `tan(x)` | Tangent | `tan(pi/4)` |
| `log(x)` | Natural logarithm | `log(e)` |
| `exp(x)` | Exponential | `exp(1)` |
| `sqrt(x)` | Square root | `sqrt(4)` |
| `abs(x)` | Absolute value | `abs(-5)` |

### Constants

| Constant | Value | Example |
|----------|-------|---------|
| `pi` | π (3.14159...) | `sin(pi/2)` |
| `e` | e (2.71828...) | `log(e)` |
| `phi` | φ (1.61803...) | `phi^2` |

### Symbolic Operations

| Operation | Syntax | Example |
|-----------|--------|---------|
| Differentiation | `deriv(expr, var)` | `deriv(x^2, x)` |
| Integration | `integrate(expr, var)` | `integrate(2*x, x)` |

## 📊 Performance Characteristics

### Execution Time

| Operation | Fast (f64) | Precise (128-bit) |
|-----------|------------|-------------------|
| Addition | 1ns | 100ns |
| Multiplication | 1ns | 150ns |
| Sine | 10ns | 1μs |
| Exponential | 20ns | 2μs |
| Complex expression | 100ns | 10μs |

### Memory Usage

| Type | Size |
|------|------|
| `Number::Fast` | 8 bytes |
| `Number::Precise(128-bit)` | ~128 bytes |
| `Number::Precise(256-bit)` | ~256 bytes |

## 🔄 Integration Examples

### Basic Expression Evaluation

```rust
let expr = "sin(pi/2) + 3^2";
let result = evaluate_expression(expr, 0);

match result {
    Ok(num) => println!("Result: {}", num.to_f64()),
    Err(e) => println!("Error: {}", e),
}
```

### Precision Comparison

```rust
let expr = "exp(1) * log(e)";

// Fast mode
let fast_result = evaluate_expression(expr, 0)?;
println!("Fast result: {}", fast_result.to_f64());

// Precise mode
let precise_result = evaluate_expression(expr, 128)?;
println!("Precise result: {}", precise_result.to_f64());

// Compare
let diff = (fast_result.to_f64() - precise_result.to_f64()).abs();
println!("Difference: {}", diff);
```

### Symbolic Mathematics

```rust
// Differentiation
let derivative = differentiate("x^2 + sin(x)", "x")?;
println!("d/dx (x^2 + sin(x)) = {}", derivative);

// Integration
let integral = integrate("2*x", "x")?;
println!("∫(2*x) dx = {}", integral);
```

### Error Estimation

```rust
let expr = "sin(pi/2) + 3^2";
let result = evaluate_expression(expr, 0)?;

// Estimate error
let f64_result = result.to_f64();
let estimated_error = f64_result.abs() * f64::EPSILON;

println!("Result: {}", f64_result);
println!("Estimated error: {}", estimated_error);
```

## 🛡️ Safety Considerations

### Overflow Handling

```rust
fn safe_add(a: Number, b: Number) -> Result<Number, String> {
    match (a, b) {
        (Number::Fast(a_val), Number::Fast(b_val)) => {
            let result = a_val + b_val;
            if result.is_infinite() && !a_val.is_infinite() && !b_val.is_infinite() {
                Err("Arithmetic overflow".to_string())
            } else {
                Ok(Number::Fast(result))
            }
        }
        #[cfg(feature = "precision")]
        (Number::Precise(a_val), Number::Precise(b_val)) => {
            Ok(Number::Precise(a_val + b_val))
        }
        // ... other cases
    }
}
```

### Underflow Handling

```rust
fn safe_mul(a: Number, b: Number) -> Result<Number, String> {
    match (a, b) {
        (Number::Fast(a_val), Number::Fast(b_val)) => {
            let result = a_val * b_val;
            if result.abs() < f64::MIN_POSITIVE && result != 0.0 {
                Ok(Number::Fast(0.0)) // Underflow to zero
            } else {
                Ok(Number::Fast(result))
            }
        }
        #[cfg(feature = "precision")]
        (Number::Precise(a_val), Number::Precise(b_val)) => {
            Ok(Number::Precise(a_val * b_val))
        }
        // ... other cases
    }
}
```

## 🚀 Future API Enhancements

### Planned Features

1. **Matrix Operations**: Linear algebra support
2. **Equation Solving**: Numerical equation solving
3. **Unit Conversion**: Integrated unit handling
4. **Advanced Symbolic Math**: Full CAS capabilities

### API Evolution

```rust
// Future: Matrix operations
fn matrix_multiply(a: Matrix, b: Matrix) -> Result<Matrix, String>;

// Future: Equation solving
fn solve_equation(equation: &str, var: &str) -> Result<Vec<Number>, String>;

// Future: Unit conversion
fn convert_units(value: Number, from: &str, to: &str) -> Result<Number, String>;
```

## 📁 File Structure

```
rust/src/
├── features/
│   ├── cas_types.rs           # Number enum and operations
│   └── math_tool.rs           # Math tool implementation
├── state.rs                  # App state with math tool
└── router.rs                 # Action routing
```

## 📚 Related Documents

- **[Math Tool Overview](../features/math-tool/overview.md)** - Math tool features
- **[Precision Implementation](../features/math-tool/precision.md)** - Precision system details
- **[Symbolic Math](../features/math-tool/symbolic.md)** - Symbolic computation
- **[System Architecture](../architecture/overview.md)** - Overall system architecture

**Last updated:** 2025-12-14