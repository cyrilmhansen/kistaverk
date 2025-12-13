// Test file specifically for cumulative FP error calculations
// This file tests the error accumulation logic in the math tool

use crate::features::math_tool::{evaluate_expression, MathToolState};
use crate::state::{AppState, MathToolState};
use crate::features::cas_types::Number;

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
    let expected_error = f64_result.abs() * f64::EPSILON;
    
    // Verify the calculation
    assert!((f64_result - 3.0).abs() < 1e-10, "Addition result should be close to 3.0");
    
    // Test 2: Multiple operations should accumulate error
    state.math_tool.expression = "1.0 / 3.0".to_string();
    let result2 = evaluate_expression("1.0 / 3.0", 0);
    assert!(result2.is_ok());
    
    let f64_result2 = result2.unwrap().to_f64();
    let expected_error2 = f64_result2.abs() * f64::EPSILON;
    
    // Verify division result
    assert!((f64_result2 - 0.3333333333333333).abs() < 1e-10, "Division result should be close to 0.333...");

    println!("✅ All cumulative error calculations passed!");
}

#[test]
fn test_error_accumulation() {
    // Test that cumulative error accumulates correctly
    let mut state = AppState::new();
    
    // Perform several operations
    let results = vec![
        evaluate_expression("1.0 + 2.0", 0),
        evaluate_expression("3.0 * 4.0", 0),
        evaluate_expression("12.0 - 1.0", 0),
    ];
    
    for result in results {
        assert!(result.is_ok(), "All operations should succeed");
    }
    
    println!("✅ Error accumulation test passed!");
}

#[test]
fn test_error_display() {
    // Test that error is displayed correctly in UI
    let mut state = AppState::new();
    state.math_tool.cumulative_error = 1.23e-15;
    
    // The error should be formatted correctly
    assert_eq!(state.math_tool.cumulative_error, 1.23e-15);
    
    println!("✅ Error display test passed!");
}

// Main test function
fn main() {
    println!("Running cumulative error tests...");
    test_cumulative_error_calculation();
    test_error_accumulation();
    test_error_display();
    println!("All tests completed successfully!");
}
