// Copyright 2025 John Doe
// SPDX-License-Identifier: MIT OR Apache-2.0

// Function Analysis UI for MIR advanced features

use crate::state::AppState;
use crate::ui::{Button as UiButton, Column as UiColumn, Text as UiText, TextInput as UiTextInput};
use serde_json::Value;
use crate::features::automatic_differentiation::ADMode;
use crate::features::c_based_ad::CBasedAutomaticDifferentiator;

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
        let error_msg = format!("Error: {}", error);
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
            // TODO: Implement performance analysis
            state.math_tool.error = Some("Performance analysis not yet implemented".to_string());
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
            // TODO: Implement plotting
            state.math_tool.error = Some("Plotting not yet implemented".to_string());
        }
        "function_analysis_stability" => {
            // TODO: Implement stability test
            state.math_tool.error = Some("Stability test not yet implemented".to_string());
        }
        _ => {}
    }
}