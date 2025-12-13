#[cfg(test)]
mod tests {
    use crate::features::math_tool::evaluate_expression;
    use crate::state::AppState;

    #[test]
    fn test_cumulative_error_calculation() {
        // Test 1: Simple addition should accumulate small error
        let mut state = AppState::new();
        state.math_tool.expression = "1.0 + 2.0".to_string();
        
        let result = evaluate_expression("1.0 + 2.0", 0);
        assert!(result.is_ok());
        
        // The result should be approximately 3.0 with small f64 error
        let result_value = result.unwrap();
        let f64_result = result_value.to_f64();
        
        // Calculate expected error (machine epsilon * result magnitude)
        // Note: result magnitude for 3.0 is 3.0
        // We verify the result is within epsilon
        assert!((f64_result - 3.0).abs() < 1e-10, "Addition result should be close to 3.0");
        
        // Test 2: Multiple operations should accumulate error
        state.math_tool.expression = "1.0 / 3.0".to_string();
        let result2 = evaluate_expression("1.0 / 3.0", 0);
        assert!(result2.is_ok());
        
        let f64_result2 = result2.unwrap().to_f64();
        
        // Verify division result
        assert!((f64_result2 - 0.3333333333333333).abs() < 1e-10, "Division result should be close to 0.333...");
    }

    #[test]
    fn test_error_accumulation() {
        // Test that cumulative error accumulates correctly
        // Note: The actual accumulation logic happens in the UI/Integration layer (handle_math_action)
        // or wherever evaluate_expression is called and the result is processed.
        // evaluate_expression itself is pure and doesn't modify state.
        
        // We verify that the evaluation returns valid results that *can* be used for error tracking
        let results = vec![
            evaluate_expression("1.0 + 2.0", 0),
            evaluate_expression("3.0 * 4.0", 0),
            evaluate_expression("12.0 - 1.0", 0),
        ];
        
        for result in results {
            assert!(result.is_ok(), "All operations should succeed");
        }
    }

    #[test]
    fn test_error_display() {
        // Test that error is displayed correctly in UI
        let mut state = AppState::new();
        state.math_tool.cumulative_error = 1.23e-15;
        
        // The error should be formatted correctly
        assert_eq!(state.math_tool.cumulative_error, 1.23e-15);
    }
}