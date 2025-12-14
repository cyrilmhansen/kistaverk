# Math Tool Overview

The Math Tool is kistaverk's primary interface for mathematical computation, providing both numerical evaluation and symbolic mathematics capabilities.

## 🧮 Features

### Basic Mathematical Operations

- **Arithmetic**: `+`, `-`, `*`, `/`, `^` (exponentiation)
- **Functions**: `sin`, `cos`, `tan`, `log`, `exp`, `sqrt`, etc.
- **Constants**: `pi`, `e`, `phi` (golden ratio)
- **Parentheses**: Full support for expression grouping

### Advanced Capabilities

- **Symbolic Differentiation**: `deriv(x^2 + sin(x), x)`
- **Symbolic Integration**: `integrate(x^2, x)`
- **Precision Control**: Toggle between fast (f64) and precise (arbitrary) modes
- **Error Estimation**: Track cumulative floating-point error

## 🎯 User Interface

```
┌─────────────────────────────────────────┐
│ Math Tool                              │
├─────────────────────────────────────────┤
│                                     │
│ Expression: [sin(pi/2) + 3^2]        │
│                                     │
│ [Calculate] [Clear history]           │
│                                     │
│ History:                              │
│ • sin(pi/2) + 3^2 = 4.0              │
│ • deriv(x^2, x) = 2*x                │
│ • integrate(x^2, x) = x^3/3          │
│                                     │
│ Backend: Standard Precision (f64)     │
│ [Use high precision (128-bit)]        │
│                                     │
└─────────────────────────────────────────┘
```

## 🔢 Expression Syntax

### Basic Expressions

```
2 + 3 * 4          # Basic arithmetic with precedence
sin(pi/2) + 3^2    # Functions and constants
(1 + 2) * (3 + 4)  # Parentheses for grouping
```

### Functions

| Function | Description | Example |
|----------|-------------|---------|
| `sin(x)` | Sine | `sin(pi/2) = 1` |
| `cos(x)` | Cosine | `cos(0) = 1` |
| `tan(x)` | Tangent | `tan(pi/4) = 1` |
| `log(x)` | Natural logarithm | `log(e) = 1` |
| `exp(x)` | Exponential | `exp(1) = e` |
| `sqrt(x)` | Square root | `sqrt(4) = 2` |
| `abs(x)` | Absolute value | `abs(-5) = 5` |

### Symbolic Operations

#### Differentiation

```
deriv(expression, variable)
```

Examples:
```
deriv(x^2 + 3*x + 2, x)    # Result: 2*x + 3
deriv(sin(x), x)            # Result: cos(x)
deriv(exp(x^2), x)          # Result: 2*x*exp(x^2)
```

#### Integration

```
integrate(expression, variable)
```

Examples:
```
integrate(2*x, x)           # Result: x^2
integrate(cos(x), x)        # Result: sin(x)
integrate(1/x, x)           # Result: log(x)
```

## ⚙️ Precision Modes

### Fast Mode (f64)

- **Default mode** for quick calculations
- **Performance**: ~15 decimal digits precision
- **Speed**: Native hardware floating-point
- **Memory**: Minimal overhead

### Precise Mode (rug::Float)

- **Arbitrary precision** for exact calculations
- **Performance**: Configurable precision (64-1024+ bits)
- **Speed**: Software-based, slower than f64
- **Memory**: Higher overhead

**Toggle between modes**: Use the "Use high precision" button in the UI

## 📊 Error Handling

### Error Types

| Error Type | Description | Example |
|------------|-------------|---------|
| Syntax Error | Invalid expression syntax | `2 + * 3` |
| Domain Error | Invalid domain | `sqrt(-1)` |
| Overflow | Result too large | `1e308 * 1e308` |
| Timeout | Calculation took too long | Complex recursive expression |

### Error Display

```
Expression: [invalid expression]
Error: Syntax error at position 3: unexpected '*'
```

## 🔧 Implementation Details

### Expression Processing Pipeline

```mermaid
flowchart TD
    A[User Input] --> B[Tokenizer]
    B --> C[Parser (Shunting-Yard)]
    C --> D[RPN Evaluator]
    D --> E[Result Formatter]
    E --> F[Display]
```

### Tokenization

Converts string input to tokens:
```rust
enum Token {
    NumberStr(String),  // "3.14", "2.5e10"
    Variable(String),   // "x", "y"
    Function(String),   // "sin", "cos"
    Operator(Op),       // +, -, *, /, ^
    LeftParen,          // (
    RightParen,         // )
    Comma,              // ,
}
```

### Parsing

Uses the Shunting-Yard algorithm to convert infix to RPN:
```
Input:  "3 + 4 * 2"
Tokens: [Number("3"), +, Number("4"), *, Number("2")]
RPN:    [3, 4, 2, *, +]
```

### Evaluation

Stack-based RPN evaluation:
```rust
fn eval_rpn(tokens: &[Token], precision: u32) -> Result<Number, String> {
    let mut stack = Vec::new();
    
    for token in tokens {
        match token {
            Token::NumberStr(s) => stack.push(parse_number(s, precision)?),
            Token::Operator(op) => {
                let b = stack.pop().ok_or("Missing operand")?;
                let a = stack.pop().ok_or("Missing operand")?;
                stack.push(apply_operator(a, b, op)?);
            }
            // ... other cases
        }
    }
    
    stack.pop().ok_or("Empty result")
}
```

## 📁 File Structure

```
rust/src/features/
├── math_tool.rs       # Main math tool implementation
├── cas_types.rs       # Number type and operations
└── math_tool_test.rs  # Unit tests
```

## 🚀 Future Enhancements

### Short-term
- **Function Library**: Predefined mathematical functions
- **Expression History**: Save and reuse previous expressions
- **Variable Support**: User-defined variables

### Medium-term
- **MIR Integration**: Use MIR JIT for complex expressions
- **Matrix Operations**: Linear algebra support
- **Plot Integration**: Visualize functions

### Long-term
- **Advanced Symbolic Math**: Full CAS capabilities
- **Equation Solving**: Numerical and symbolic solvers
- **Unit Conversion**: Integrated unit handling

## 📚 Related Documents

- **[Precision Implementation](precision.md)** - Detailed precision system
- **[Symbolic Math](symbolic.md)** - Symbolic computation details
- **[System Architecture](../../architecture/overview.md)** - Overall architecture
- **[CAS Design](../../architecture/cas-design.md)** - CAS architecture

**Last updated:** 2025-12-14