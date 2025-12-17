// Copyright 2025 John Doe
// SPDX-License-Identifier: MIT OR Apache-2.0

// Function Analysis UI for MIR advanced features

use crate::state::AppState;
use crate::ui::{Button as UiButton, Column as UiColumn, Text as UiText, TextInput as UiTextInput};
use serde_json::Value;
use crate::features::automatic_differentiation::ADMode;
use crate::features::c_based_ad::CBasedAutomaticDifferentiator;
use std::time::Instant;

pub fn render_function_analysis_screen(state: &AppState) -> Value {
    let title = "Function Analysis";
    let description = "Advanced MIR-based function analysis and visualization";
    let ad_mode_text = "AD Mode:";
    
    let forward_label = if state.math_tool.get_ad_mode() == ADMode::Forward { "Forward Mode (✓)" } else { "Forward Mode" };
    let reverse_label = if state.math_tool.get_ad_mode() == ADMode::Reverse { "Reverse Mode (✓)" } else { "Reverse Mode" };
    
    let mut children = vec![
        serde_json::to_value(UiText::new(title).size(20.0)).unwrap(),
        serde_json::to_value(
            UiText::new(description)
                .size(14.0),
        )
        .unwrap(),
        serde_json::to_value(
            UiTextInput::new("function_analysis_expr")
                .hint("Enter function to analyze (e.g., x^2 + sin(x))")
                .text(&state.math_tool.expression)
                .single_line(true)
                .debounce_ms(150),
        )
        .unwrap(),
        serde_json::to_value(UiText::new(ad_mode_text).size(14.0)).unwrap(),
        serde_json::to_value(UiButton::new(forward_label, "function_analysis_set_forward")).unwrap(),
        serde_json::to_value(UiButton::new(reverse_label, "function_analysis_set_reverse")).unwrap(),
        serde_json::to_value(UiButton::new("Analyze Performance", "function_analysis_performance")).unwrap(),
        serde_json::to_value(UiButton::new("Compute Derivative", "function_analysis_derivative")).unwrap(),
        serde_json::to_value(UiButton::new("Plot Function", "function_analysis_plot")).unwrap(),
        serde_json::to_value(UiButton::new("Stability Test", "function_analysis_stability")).unwrap(),
    ];

    // Show analysis results if available
    if let Some(error) = &state.math_tool.error {
        let error_msg = format!("Result: {}", error);
        children.push(
            serde_json::to_value(UiText::new(&error_msg).size(12.0)).unwrap(),
        );
    }

    // Add visualization placeholder
    {
        let viz_text = "Visualization will appear here";
        children.push(
            serde_json::to_value(UiText::new(viz_text).size(14.0)).unwrap(),
        );
    }

    serde_json::to_value(UiColumn::new(children)).unwrap()
}

pub fn handle_function_analysis_action(state: &mut AppState, action: &str) {
    match action {
        "function_analysis_set_forward" => {
            state.math_tool.set_ad_mode(ADMode::Forward);
        }
        "function_analysis_set_reverse" => {
            state.math_tool.set_ad_mode(ADMode::Reverse);
        }
        "function_analysis_performance" => {
            let mut c_ad = CBasedAutomaticDifferentiator::new();
            let expr = if state.math_tool.expression.is_empty() {
                "x^2 - cos(x)".to_string()
            } else {
                state.math_tool.expression.clone()
            };
            
            let start = Instant::now();
            match c_ad.differentiate(&expr, "x") {
                Ok(func_name) => {
                    // Run 50 iterations (C compilation takes time, so keep it low for demo)
                    for _ in 0..50 {
                         let _ = c_ad.evaluate_derivative(&func_name, 1.0);
                    }
                    let duration = start.elapsed();
                    state.math_tool.error = Some(format!("Benchmark: 50 ops in {:.2?}", duration));
                }
                Err(e) => {
                    state.math_tool.error = Some(format!("AD failed: {}", e));
                }
            }
        }
        "function_analysis_derivative" => {
            // Use C-based automatic differentiation
            let mut c_ad = CBasedAutomaticDifferentiator::new();
            
            // Get the expression from the UI state
            let expr = if state.math_tool.expression.is_empty() {
                "x^2 - cos(x)".to_string() // Default expression for testing
            } else {
                state.math_tool.expression.clone()
            };
            
            // Try to compute the derivative using C-based AD
            match c_ad.differentiate(&expr, "x") {
                Ok(func_name) => {
                    // Evaluate the derivative at x=1
                    match c_ad.evaluate_derivative(&func_name, 1.0) {
                        Ok(derivative_value) => {
                            state.math_tool.error = Some(format!("Derivative at x=1: {}", derivative_value.to_f64()));
                        }
                        Err(e) => {
                            state.math_tool.error = Some(format!("Evaluation failed: {}", e));
                        }
                    }
                }
                Err(e) => {
                    state.math_tool.error = Some(format!("Differentiation failed: {}", e));
                }
            }
        }
        "function_analysis_plot" => {
             let mut c_ad = CBasedAutomaticDifferentiator::new();
             let expr = if state.math_tool.expression.is_empty() {
                "x^2 - cos(x)".to_string()
            } else {
                state.math_tool.expression.clone()
            };
            
            match c_ad.differentiate(&expr, "x") {
                Ok(func_name) => {
                    let mut csv = String::from("x,derivative\n");
                    // Generate 20 points from -5 to 5
                    for i in 0..21 {
                        let x = -5.0 + (i as f64) * 0.5;
                        if let Ok(val) = c_ad.evaluate_derivative(&func_name, x) {
                            csv.push_str(&format!("{},{}\n", x, val.to_f64()));
                        }
                    }
                    let path = "/tmp/kistaverk_plot.csv";
                    match std::fs::write(path, csv) {
                        Ok(_) => state.math_tool.error = Some(format!("Plot data saved to {}", path)),
                        Err(e) => state.math_tool.error = Some(format!("Failed to write plot: {}", e)),
                    }
                }
                Err(e) => state.math_tool.error = Some(format!("AD failed: {}", e)),
            }
        }
        "function_analysis_stability" => {
             let mut c_ad = CBasedAutomaticDifferentiator::new();
             let expr = if state.math_tool.expression.is_empty() {
                "x^2 - cos(x)".to_string()
            } else {
                state.math_tool.expression.clone()
            };
            
            match c_ad.differentiate(&expr, "x") {
                Ok(func_name) => {
                    let points = [0.0, 1.0, -1.0, 100.0];
                    let mut stable = true;
                    for p in points {
                        if let Ok(val) = c_ad.evaluate_derivative(&func_name, p) {
                            if val.to_f64().is_nan() || val.to_f64().is_infinite() {
                                stable = false;
                                state.math_tool.error = Some(format!("Unstable at x={}", p));
                                break;
                            }
                        } else {
                             stable = false;
                             state.math_tool.error = Some(format!("Eval failed at x={}", p));
                             break;
                        }
                    }
                    if stable {
                        state.math_tool.error = Some("Stability Test Passed".to_string());
                    }
                }
                Err(e) => state.math_tool.error = Some(format!("AD failed: {}", e)),
            }
        }
        _ => {}
    }
}