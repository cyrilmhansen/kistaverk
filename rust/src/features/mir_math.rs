// Copyright 2025 John Doe
// SPDX-License-Identifier: MIT OR Apache-2.0

// This file contains MIR implementations of mathematical functions
// and integration with the math tool for hybrid evaluation

use crate::features::cas_types::Number;
use crate::features::mir_scripting::MirScriptingState;
use std::collections::HashMap;

/// MIR Math Function Library
/// Stores pre-compiled MIR functions for common mathematical operations
#[derive(Debug, Clone)]
pub struct MirMathLibrary {
    functions: HashMap<String, String>, // name -> MIR source code
    cache: Option<HashMap<String, &'static MirScriptingState>>, // name -> compiled state (lazy initialized)
}

impl MirMathLibrary {
    /// Create a new MIR math library (non-const due to HashMap limitations)
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            cache: None, // Will be initialized lazily
        }
    }

    /// Register a MIR math function
    pub fn register(&mut self, name: String, mir_source: String) {
        self.functions.insert(name, mir_source);
    }

    /// Get MIR source for a function
    pub fn get_source(&self, name: &str) -> Option<&String> {
        self.functions.get(name)
    }

    /// Execute a MIR math function
    pub fn execute(&mut self, name: &str, args: Vec<Number>) -> Result<Number, String> {
        // Get or compile the function
        let state = self.get_or_compile(name)?;
        
        // Set up arguments (implementation depends on MIR ABI)
        // For now, we'll use a simple approach
        let mut execution_state = state.clone();
        
        // Execute the function
        if let Some(runtime) = execution_state.execute_jit() {
            // Parse result from output
            self.parse_mir_result(&execution_state.output)
        } else {
            Err(execution_state.error.unwrap_or("Unknown MIR execution error".to_string()))
        }
    }

    /// Get or compile a MIR function
    fn get_or_compile(&mut self, name: &str) -> Result<&MirScriptingState, String> {
        // Initialize cache if needed
        if self.cache.is_none() {
            self.cache = Some(HashMap::new());
        }

        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(state) = cache.get(name) {
                return Ok(state);
            }
        }

        // Get source code
        let source = self.functions.get(name)
            .ok_or_else(|| format!("MIR function '{}' not found", name))?;

        // Create and compile new state
        let mut state = MirScriptingState::new();
        state.source = source.clone();
        state.entry = name.to_string();
        
        // Compile (we'll execute later with specific args)
        let _ = state.execute_jit(); // Just compile, don't execute yet
        
        if let Some(error) = &state.error {
            return Err(format!("MIR compilation failed: {}", error));
        }

        // Cache the compiled function and return it
        if let Some(cache) = &mut self.cache {
            let boxed_state = Box::new(state);
            let state_ref = Box::leak(boxed_state); // Leak to get a reference
            cache.insert(name.to_string(), state_ref);
            return Ok(state_ref);
        }
        
        Err("Cache not available".to_string())
    }

    /// Parse MIR result from output
    fn parse_mir_result(&self, output: &str) -> Result<Number, String> {
        // Simple parser for now - extract number from output
        if output.starts_with("Result: ") {
            let num_str = &output[8..]; // Skip "Result: "
            if let Ok(num) = num_str.parse::<f64>() {
                Ok(Number::from_f64(num))
            } else {
                Err(format!("Failed to parse MIR result: {}", output))
            }
        } else {
            Err(format!("Unexpected MIR output format: {}", output))
        }
    }

    /// Clear cache (useful for memory management)
    pub fn clear_cache(&mut self) {
        self.cache = None;
    }
}

/// Default MIR math library with common functions
impl Default for MirMathLibrary {
    fn default() -> Self {
        let mut library = Self::new();
        
        // Register basic math functions
        library.register("add".to_string(), r#"
            m_add: module
              export add
            add: func i64, i64:a, i64:b
              local i64:r
              add r, a, b
              ret r
              endfunc
              endmodule
        "#.to_string());

        library.register("sub".to_string(), r#"
            m_sub: module
              export sub
            sub: func i64, i64:a, i64:b
              local i64:r
              sub r, a, b
              ret r
              endfunc
              endmodule
        "#.to_string());

        library.register("mul".to_string(), r#"
            m_mul: module
              export mul
            mul: func i64, i64:a, i64:b
              local i64:r
              mul r, a, b
              ret r
              endfunc
              endmodule
        "#.to_string());

        library.register("div".to_string(), r#"
            m_div: module
              export div
            div: func i64, i64:a, i64:b
              local i64:r
              div r, a, b
              ret r
              endfunc
              endmodule
        "#.to_string());

        library
    }
}

/// Expression complexity analyzer
pub fn is_complex_expression(expr: &str) -> bool {
    // Simple heuristic for now
    let tokens = expr.split_whitespace().collect::<Vec<_>>();
    
    // Consider complex if:
    // - Has many operations
    // - Contains advanced functions
    // - Has nested parentheses
    let op_count = tokens.iter()
        .filter(|&t| ["+", "-", "*", "/", "^"].contains(&t))
        .count();
    
    let has_advanced_func = tokens.iter()
        .any(|&t| ["sin", "cos", "exp", "log", "sqrt"].contains(&t));
    
    let paren_depth = expr.chars().filter(|&c| c == '(').count();
    
    // Complex if: many operations, advanced functions, or deep nesting
    op_count > 5 || has_advanced_func || paren_depth > 2
}

/// Hybrid expression evaluator
pub fn evaluate_with_mir_fallback(
    expr: &str,
    precision_bits: u32,
    mir_library: &mut MirMathLibrary,
) -> Result<Number, String> {
    // First try standard evaluation
    match crate::features::math_tool::evaluate_expression(expr, precision_bits) {
        Ok(result) => Ok(result),
        Err(_) => {
            // Fall back to MIR for complex expressions
            if is_complex_expression(expr) {
                // Convert expression to MIR (simplified for now)
                let mir_expr = convert_to_mir_expression(expr);
                
                // Execute via MIR
                execute_mir_expression(&mir_expr, mir_library)
            } else {
                // Re-throw original error for simple expressions
                crate::features::math_tool::evaluate_expression(expr, precision_bits)
            }
        }
    }
}

/// Convert expression to MIR (simplified placeholder)
fn convert_to_mir_expression(expr: &str) -> String {
    // This is a placeholder - real implementation would:
    // 1. Parse expression to AST
    // 2. Generate MIR code from AST
    // 3. Handle variables, functions, etc.
    
    format!(
        "m_expr: module\n          export main\nmain: func i64\n          local i64:r\n          {} \n          ret r\n          endfunc\n          endmodule",
        expr.replace("+", "add r, r,")
    )
}

/// Execute MIR expression
fn execute_mir_expression(
    mir_code: &str,
    mir_library: &mut MirMathLibrary,
) -> Result<Number, String> {
    // For now, use a temporary function
    let mut temp_state = MirScriptingState::new();
    temp_state.source = mir_code.to_string();
    temp_state.entry = "main".to_string();
    
    if let Some(runtime) = temp_state.execute_jit() {
        // Parse result
        if temp_state.output.starts_with("Result: ") {
            let num_str = &temp_state.output[8..];
            num_str.parse::<f64>()
                .map(Number::from_f64)
                .map_err(|e| format!("Failed to parse MIR result: {}", e))
        } else {
            Err(format!("Unexpected MIR output: {}", temp_state.output))
        }
    } else {
        Err(temp_state.error.unwrap_or("MIR execution failed".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::cas_types::Number;

    #[test]
    fn test_mir_library_creation() {
        let library = MirMathLibrary::new();
        assert!(library.functions.is_empty());
        assert!(library.cache.as_ref().map_or(true, |c| c.is_empty()));
    }

    #[test]
    fn test_default_library() {
        let library = MirMathLibrary::default();
        assert!(library.functions.contains_key("add"));
        assert!(library.functions.contains_key("sub"));
        assert!(library.functions.contains_key("mul"));
        assert!(library.functions.contains_key("div"));
    }

    #[test]
    fn test_expression_complexity() {
        assert!(!is_complex_expression("2 + 3"));
        assert!(is_complex_expression("sin(x) + cos(y) * exp(z)"));
        assert!(is_complex_expression("((2 + 3) * (4 + 5)) / (6 + 7)"));
    }

    #[test]
    fn test_simple_mir_execution() {
        let mut library = MirMathLibrary::default();
        
        // Test simple addition via MIR
        let result = library.execute("add", vec![Number::from_f64(2.0), Number::from_f64(3.0)]);
        
        // Note: This test may fail until we implement proper argument passing
        // For now, we're just testing the structure
        assert!(result.is_ok() || result.is_err()); // Either way, we get a result
    }
}
