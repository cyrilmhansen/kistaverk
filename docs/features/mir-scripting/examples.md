# MIR Scripting Examples

This document provides practical examples of MIR programming for kistaverk, demonstrating various algorithms and techniques.

## 🧮 Mathematical Functions

### Factorial

```mir
m_fact:    module
           export fact

fact:      func i64, i64:n
           local i64:result, i64:i
           mov result, 1
           mov i, 1

loop:      bgt done, i, n
           mul result, result, i
           add i, i, 1
           jmp loop

done:      ret result
           endfunc
           endmodule
```

**Usage**: `fact(5)` → **Result**: `120`

### Greatest Common Divisor (GCD)

```mir
m_gcd:     module
           export gcd

gcd:       func i64, i64:a, i64:b
           local i64:temp

loop:      beq done, b, 0
           mov temp, b
           mod b, a, b
           mov a, temp
           jmp loop

done:      ret a
           endfunc
           endmodule
```

**Usage**: `gcd(48, 18)` → **Result**: `6`

### Exponentiation

```mir
m_pow:     module
           export pow

pow:       func i64, i64:base, i64:exp
           local i64:result, i64:i
           mov result, 1
           mov i, 0

loop:      bge done, i, exp
           mul result, result, base
           add i, i, 1
           jmp loop

done:      ret result
           endfunc
           endmodule
```

**Usage**: `pow(2, 10)` → **Result**: `1024`

## 🔢 Algorithms

### Binary Search

```mir
m_search:  module
           export binary_search

binary_search: func i64, p:array, i64:size, i64:target
               local i64:low, i64:high, i64:mid, i64:value
               mov low, 0
               sub high, size, 1

loop:       bgt not_found, low, high
               add mid, low, high
               div mid, mid, 2
               mov value, i64:(array, mid)
               
               beq found, value, target
               blt value, target, higher
               bgt value, target, lower

higher:     add low, mid, 1
           jmp loop

lower:      sub high, mid, 1
           jmp loop

found:      ret mid

not_found:  ret -1
           endfunc
           endmodule
```

**Usage**: `binary_search(array, 10, 42)` → **Result**: `index of 42 or -1`

### Bubble Sort

```mir
m_sort:    module
           export bubble_sort

bubble_sort: func i64, p:array, i64:size
             local i64:i, i64:j, i64:temp, i64:a, i64:b
             mov i, 0

outer:      bge done, i, size
             mov j, 0

inner:      bge next_i, j, size
             sub j, j, i
             sub j, j, 1
             mov a, i64:(array, j)
             add j, j, 1
             mov b, i64:(array, j)
             blt a, b, no_swap
             
             mov temp, a
             mov i64:(array, j), temp
             sub j, j, 1
             mov i64:(array, j), b
             add j, j, 1

no_swap:    add j, j, 1
           jmp inner

next_i:     add i, i, 1
           jmp outer

done:       ret 0
           endfunc
           endmodule
```

**Usage**: `bubble_sort(array, 10)` → **Result**: `0` (array sorted in-place)

## 📊 Numerical Algorithms

### Newton's Method (Square Root)

```mir
m_newton:  module
           export sqrt

sqrt:      func f64, f64:n
           local f64:x, f64:prev, f64:diff
           mov x, n
           mov prev, 0
           mov diff, 1

loop:      blt done, diff, 0.0001
           mov prev, x
           div x, n, x
           add x, x, prev
           div x, x, 2.0
           sub diff, prev, x
           abs diff, diff
           jmp loop

done:      ret x
           endfunc
           endmodule
```

**Usage**: `sqrt(25.0)` → **Result**: `5.0`

### Numerical Integration (Trapezoidal Rule)

```mir
m_integrate: module
             export integrate

integrate:   func f64, f64:a, f64:b, i64:n
             local f64:h, f64:sum, f64:x, i64:i
             sub h, b, a
             div h, h, n
             mov sum, 0
             mov i, 1

loop:        bge done, i, n
             mul x, h, i
             add x, x, a
             
             // Call external function f(x)
             call f, sum, x
             add sum, sum, sum
             
             add i, i, 1
             jmp loop

done:        mul sum, sum, h
             add sum, sum, sum
             div sum, sum, 2.0
             ret sum
             endfunc
             endmodule
```

**Usage**: `integrate(0.0, 1.0, 1000)` → **Result**: `approximate integral`

## 🔤 String Processing

### String Length

```mir
m_strlen:  module
           export strlen

strlen:    func i64, p:str
           local i64:len
           mov len, 0

loop:      beq done, u8:(str, len), 0
           add len, len, 1
           jmp loop

done:      ret len
           endfunc
           endmodule
```

**Usage**: `strlen("hello")` → **Result**: `5`

### String Copy

```mir
m_strcpy:  module
           export strcpy

strcpy:    func i64, p:dest, p:src
           local i64:i, i64:ch
           mov i, 0

loop:      mov ch, u8:(src, i)
           mov u8:(dest, i), ch
           beq done, ch, 0
           add i, i, 1
           jmp loop

done:      ret i
           endfunc
           endmodule
```

**Usage**: `strcpy(buffer, "hello")` → **Result**: `5` (bytes copied)

## 🎯 Performance Benchmarks

### Fibonacci Performance

```mir
m_bench:   module
           export fib_bench

fib_bench: func i64, i64:n, i64:iterations
           local i64:i, i64:result
           mov i, 0
           mov result, 0

loop:      bge done, i, iterations
           call fib, result, n
           add i, i, 1
           jmp loop

done:      ret result
           endfunc
           endmodule
```

**Benchmark**: `fib_bench(20, 1000)` → **Measure**: Execution time for 1000 iterations

### Sieve Performance

```mir
m_bench:   module
           export sieve_bench

sieve_bench: func i64, i64:iterations
             local i64:i, i64:result
             mov i, 0
             mov result, 0

loop:      bge done, i, iterations
             call sieve, result, 100
             add i, i, 1
             jmp loop

done:      ret result
           endfunc
           endmodule
```

**Benchmark**: `sieve_bench(100)` → **Measure**: Execution time for 100 sieve iterations

## 🔧 Advanced Examples

### Recursive Factorial

```mir
m_fact_rec: module
            export fact

fact:       func i64, i64:n
            local i64:result
            beq base, n, 0
            sub n, n, 1
            call fact, result, n
            mul result, result, n
            add n, n, 1
            ret result

base:       ret 1
            endfunc
            endmodule
```

**Usage**: `fact(5)` → **Result**: `120`

### Matrix Multiplication

```mir
m_matrix:  module
           export matmul

matmul:    func i64, p:a, p:b, p:c, i64:size
           local i64:i, i64:j, i64:k
           local i64:sum, i64:a_val, i64:b_val
           mov i, 0

outer_i:   bge done, i, size
           mov j, 0

outer_j:   bge next_i, j, size
           mov sum, 0
           mov k, 0

inner:     bge store, k, size
           mul a_val, i, size
           add a_val, a_val, k
           mov a_val, i64:(a, a_val)
           
           mul b_val, k, size
           add b_val, b_val, j
           mov b_val, i64:(b, b_val)
           
           mul a_val, a_val, b_val
           add sum, sum, a_val
           
           add k, k, 1
           jmp inner

store:     mul a_val, i, size
           add a_val, a_val, j
           mov i64:(c, a_val), sum
           
           add j, j, 1
           jmp outer_j

next_i:    add i, i, 1
           jmp outer_i

done:      ret 0
           endfunc
           endmodule
```

**Usage**: `matmul(a, b, c, 3)` → **Result**: `0` (result stored in c)

## 📚 Example Library

### Basic Examples

- **Arithmetic**: Addition, subtraction, multiplication, division
- **Control Flow**: If-else, loops, switches
- **Functions**: Function calls, recursion
- **Memory**: Array operations, pointer arithmetic

### Mathematical Examples

- **Algebra**: Polynomial evaluation, root finding
- **Calculus**: Numerical differentiation and integration
- **Linear Algebra**: Matrix operations, vector math
- **Statistics**: Mean, variance, standard deviation

### Algorithm Examples

- **Sorting**: Bubble sort, quicksort, mergesort
- **Searching**: Binary search, linear search
- **Graph Algorithms**: BFS, DFS (future)
- **Dynamic Programming**: Fibonacci, knapsack (future)

## 🚀 Creating Your Own Examples

### Example Template

```mir
m_example:  module
            export main

main:       func i64
            local i64:result
            
            # Your code here
            mov result, 42
            
            ret result
            endfunc
            endmodule
```

### Best Practices

1. **Use Descriptive Names**: Clear function and variable names
2. **Add Comments**: Explain complex logic
3. **Handle Edge Cases**: Check for zero, negative numbers, etc.
4. **Optimize Loops**: Minimize operations inside loops
5. **Use Functions**: Break complex logic into functions

### Testing Examples

```rust
// Test MIR examples in kistaverk
let mut mir_state = MirScriptingState::new();
mir_state.source = "m_example: module...".to_string();
mir_state.entry = "main".to_string();

let runtime = mir_state.execute_jit();
println!("Result: {}, Time: {:?}ms", mir_state.output, runtime);
```

## 📁 File Structure

```
rust/src/
├── features/
│   └── mir_scripting.rs       # Main MIR scripting implementation
├── mir_tests.rs              # MIR integration tests
└── examples/                 # Example MIR programs (future)
    ├── math.mir               # Mathematical examples
    ├── algorithms.mir         # Algorithm examples
    └── benchmarks.mir         # Performance benchmarks
```

## 📚 Related Documents

- **[MIR Scripting Overview](overview.md)** - MIR scripting features
- **[MIR Integration](integration.md)** - Integration with other features
- **[MIR JIT Integration](../../architecture/mir-integration.md)** - MIR architecture details
- **[System Architecture](../../architecture/overview.md)** - Overall system architecture

**Last updated:** 2025-12-14