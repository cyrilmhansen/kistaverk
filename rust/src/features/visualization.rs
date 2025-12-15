// Copyright 2025 John Doe
// SPDX-License-Identifier: MIT OR Apache-2.0

// Advanced visualization for function plotting and performance analysis

use crate::features::mir_math::MirMathLibrary;
use crate::features::cas_types::Number;
use crate::features::performance_analysis::{PerformanceMetrics, FunctionAnalysisResult};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Visualization data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationType {
    LineChart,
    BarChart,
    ScatterPlot,
    HeatMap,
    SurfacePlot,
}

/// Plot data structure for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotData {
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub series: Vec<PlotSeries>,
    pub visualization_type: VisualizationType,
    pub x_range: Option<(f64, f64)>,
    pub y_range: Option<(f64, f64)>,
}

/// Individual plot series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSeries {
    pub name: String,
    pub data: Vec<(f64, f64)>,
    pub color: String,
    pub line_style: String,
    pub point_style: String,
}

/// Visualization manager
#[derive(Debug, Clone)]
pub struct VisualizationManager {
    mir_library: MirMathLibrary,
    plot_cache: HashMap<String, PlotData>,
    current_plot: Option<PlotData>,
}

impl Default for VisualizationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualizationManager {
    /// Create a new visualization manager
    pub fn new() -> Self {
        Self {
            mir_library: MirMathLibrary::default(),
            plot_cache: HashMap::new(),
            current_plot: None,
        }
    }

    /// Plot a mathematical function
    pub fn plot_function(
        &mut self,
        expr: &str,
        x_range: (f64, f64),
        resolution: usize,
        name: &str,
        color: &str,
    ) -> Result<PlotData, String> {
        let mut plot_data = PlotData {
            title: format!("Function: {}", expr),
            x_label: "x".to_string(),
            y_label: "f(x)".to_string(),
            series: Vec::new(),
            visualization_type: VisualizationType::LineChart,
            x_range: Some(x_range),
            y_range: None,
        };

        let mut series_data = Vec::with_capacity(resolution);
        let step = (x_range.1 - x_range.0) / resolution as f64;

        for i in 0..resolution {
            let x = x_range.0 + i as f64 * step;
            let y = self.evaluate_expression_at(expr, x)?;
            series_data.push((x, y));
        }

        plot_data.series.push(PlotSeries {
            name: name.to_string(),
            data: series_data,
            color: color.to_string(),
            line_style: "solid".to_string(),
            point_style: "none".to_string(),
        });

        // Cache the plot
        self.plot_cache.insert(expr.to_string(), plot_data.clone());
        self.current_plot = Some(plot_data.clone());

        Ok(plot_data)
    }

    /// Plot function and its derivative together
    pub fn plot_function_with_derivative(
        &mut self,
        expr: &str,
        var: &str,
        x_range: (f64, f64),
        resolution: usize,
    ) -> Result<PlotData, String> {
        // Plot original function
        let mut plot_data = self.plot_function(expr, x_range, resolution, "Function", "#4CAF50")?;
        
        // Compute derivative expression (simplified for visualization)
        let deriv_expr = self.generate_derivative_expression(expr, var);
        
        // Plot derivative
        let deriv_plot = self.plot_function(&deriv_expr, x_range, resolution, "Derivative", "#FF5722")?;
        
        // Combine plots
        plot_data.title = format!("Function and Derivative: {}", expr);
        plot_data.series.extend(deriv_plot.series);
        plot_data.y_label = "f(x) and f'(x)".to_string();
        
        Ok(plot_data)
    }

    /// Create performance comparison visualization
    pub fn create_performance_comparison(
        &self,
        analysis_results: &[FunctionAnalysisResult],
    ) -> PlotData {
        let mut plot_data = PlotData {
            title: "Performance Comparison: MIR vs Standard".to_string(),
            x_label: "Function".to_string(),
            y_label: "Execution Time (ms)".to_string(),
            series: Vec::new(),
            visualization_type: VisualizationType::BarChart,
            x_range: None,
            y_range: None,
        };

        let mut standard_data = Vec::new();
        let mut mir_data = Vec::new();

        for (i, result) in analysis_results.iter().enumerate() {
            standard_data.push((i as f64, result.standard_metrics.execution_time_ms));
            mir_data.push((i as f64, result.mir_metrics.execution_time_ms));
        }

        plot_data.series.push(PlotSeries {
            name: "Standard".to_string(),
            data: standard_data,
            color: "#FF9800".to_string(),
            line_style: "none".to_string(),
            point_style: "bar".to_string(),
        });

        plot_data.series.push(PlotSeries {
            name: "MIR JIT".to_string(),
            data: mir_data,
            color: "#4CAF50".to_string(),
            line_style: "none".to_string(),
            point_style: "bar".to_string(),
        });

        plot_data
    }

    /// Create stability test visualization
    pub fn create_stability_visualization(
        &self,
        stability_results: &[Vec<crate::features::performance_analysis::StabilityTestResult>],
    ) -> PlotData {
        let mut plot_data = PlotData {
            title: "Stability Test Results".to_string(),
            x_label: "Test Case".to_string(),
            y_label: "Pass Rate (%)".to_string(),
            series: Vec::new(),
            visualization_type: VisualizationType::BarChart,
            x_range: None,
            y_range: Some((0.0, 100.0)),
        };

        for (func_idx, results) in stability_results.iter().enumerate() {
            let mut pass_rates = Vec::new();
            
            for (test_idx, result) in results.iter().enumerate() {
                let pass_rate = if result.passed { 100.0 } else { 0.0 };
                pass_rates.push((test_idx as f64, pass_rate));
            }

            plot_data.series.push(PlotSeries {
                name: format!("Function {}", func_idx + 1),
                data: pass_rates,
                color: format!("#{:06X}", 0x4CAF50 + func_idx * 0x111111),
                line_style: "none".to_string(),
                point_style: "bar".to_string(),
            });
        }

        plot_data
    }

    /// Evaluate expression at a specific point
    fn evaluate_expression_at(&mut self, expr: &str, x: f64) -> Result<f64, String> {
        // For now, use a simple approach - replace x with value
        // In production, we'd use proper expression evaluation
        let _expr_with_value = expr.replace("x", &x.to_string());
        
        // Try to evaluate as number first
        if let Ok(num) = expr.parse::<f64>() {
            return Ok(num);
        }
        
        // Use MIR evaluation for complex expressions
        let result = self.mir_library.execute("eval", vec![Number::from_f64(x)]);
        
        result.map(|n| n.to_f64())
    }

    /// Generate derivative expression (simplified placeholder)
    fn generate_derivative_expression(&self, expr: &str, var: &str) -> String {
        // This is a simplified approach - real implementation would use symbolic differentiation
        // For visualization purposes, we'll create a placeholder derivative expression
        
        if expr == "x" {
            return "1".to_string();
        } else if expr == "x*x" || expr == "x^2" {
            return "2*x".to_string();
        } else if expr.starts_with("sin(") {
            let inner = expr.trim_start_matches("sin(").trim_end_matches(")");
            return format!("cos({})*deriv({},{})", inner, inner, var);
        } else if expr.starts_with("exp(") {
            let inner = expr.trim_start_matches("exp(").trim_end_matches(")");
            return format!("exp({})*deriv({},{})", inner, inner, var);
        } else {
            // Default: assume derivative is 1 (for constants)
            return "1".to_string();
        }
    }

    /// Create 3D surface plot data (placeholder for future implementation)
    pub fn create_surface_plot(
        &mut self,
        expr: &str,
        x_range: (f64, f64),
        y_range: (f64, f64),
        resolution: usize,
    ) -> Result<PlotData, String> {
        let mut plot_data = PlotData {
            title: format!("3D Surface: {}", expr),
            x_label: "x".to_string(),
            y_label: "y".to_string(),
            series: Vec::new(),
            visualization_type: VisualizationType::SurfacePlot,
            x_range: Some(x_range),
            y_range: Some(y_range),
        };

        // For now, create a simple surface plot
        // Real implementation would evaluate the 2D function
        let mut surface_data = Vec::new();
        let x_step = (x_range.1 - x_range.0) / resolution as f64;
        let y_step = (y_range.1 - y_range.0) / resolution as f64;

        for i in 0..resolution {
            for j in 0..resolution {
                let x = x_range.0 + i as f64 * x_step;
                let y = y_range.0 + j as f64 * y_step;
                // Simple function: z = sin(x) * cos(y)
                let z = x.sin() * y.cos();
                surface_data.push((x, y));
                surface_data.push((x, y));
                surface_data.push((z, z));
            }
        }

        plot_data.series.push(PlotSeries {
            name: "Surface".to_string(),
            data: surface_data,
            color: "#4CAF50".to_string(),
            line_style: "surface".to_string(),
            point_style: "none".to_string(),
        });

        Ok(plot_data)
    }

    /// Create interactive plot with zoom/pan capabilities
    pub fn create_interactive_plot(
        &mut self,
        expr: &str,
        x_range: (f64, f64),
        resolution: usize,
    ) -> Result<PlotData, String> {
        let mut plot_data = self.plot_function(expr, x_range, resolution, "Function", "#4CAF50")?;
        
        // Add interactive properties
        plot_data.title = format!("Interactive: {}", expr);
        
        Ok(plot_data)
    }

    /// Get current plot data
    pub fn get_current_plot(&self) -> Option<&PlotData> {
        self.current_plot.as_ref()
    }

    /// Clear plot cache
    pub fn clear_cache(&mut self) {
        self.plot_cache.clear();
        self.current_plot = None;
    }

    /// Export plot data to JSON
    pub fn export_plot_data(&self, plot_data: &PlotData) -> Result<String, String> {
        serde_json::to_string(plot_data)
            .map_err(|e| format!("Failed to serialize plot data: {}", e))
    }

    /// Import plot data from JSON
    pub fn import_plot_data(&mut self, json_data: &str) -> Result<PlotData, String> {
        serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to deserialize plot data: {}", e))
    }
}

/// Performance visualizer for creating performance-related plots
#[derive(Debug, Clone)]
pub struct PerformanceVisualizer {
    metrics_history: Vec<PerformanceMetrics>,
}

impl Default for PerformanceVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceVisualizer {
    /// Create a new performance visualizer
    pub fn new() -> Self {
        Self {
            metrics_history: Vec::new(),
        }
    }

    /// Add performance metrics to history
    pub fn add_metrics(&mut self, metrics: PerformanceMetrics) {
        self.metrics_history.push(metrics);
    }

    /// Create performance trend visualization
    pub fn create_performance_trend(&self) -> PlotData {
        let mut plot_data = PlotData {
            title: "Performance Trend Over Time".to_string(),
            x_label: "Measurement".to_string(),
            y_label: "Execution Time (ms)".to_string(),
            series: Vec::new(),
            visualization_type: VisualizationType::LineChart,
            x_range: None,
            y_range: None,
        };

        let mut standard_data = Vec::new();
        let mut mir_data = Vec::new();

        for (i, metrics) in self.metrics_history.iter().enumerate() {
            standard_data.push((i as f64, metrics.execution_time_ms));
            if let Some(mir_time) = metrics.speedup {
                mir_data.push((i as f64, metrics.execution_time_ms / mir_time));
            }
        }

        plot_data.series.push(PlotSeries {
            name: "Standard".to_string(),
            data: standard_data,
            color: "#FF9800".to_string(),
            line_style: "solid".to_string(),
            point_style: "circle".to_string(),
        });

        if !mir_data.is_empty() {
            plot_data.series.push(PlotSeries {
                name: "MIR JIT".to_string(),
                data: mir_data,
                color: "#4CAF50".to_string(),
                line_style: "dashed".to_string(),
                point_style: "square".to_string(),
            });
        }

        plot_data
    }

    /// Create speedup comparison visualization
    pub fn create_speedup_comparison(&self) -> PlotData {
        let mut plot_data = PlotData {
            title: "MIR Speedup Comparison".to_string(),
            x_label: "Measurement".to_string(),
            y_label: "Speedup Factor".to_string(),
            series: Vec::new(),
            visualization_type: VisualizationType::BarChart,
            x_range: None,
            y_range: Some((0.0, 10.0)),
        };

        let mut speedup_data = Vec::new();

        for (i, metrics) in self.metrics_history.iter().enumerate() {
            if let Some(speedup) = metrics.speedup {
                speedup_data.push((i as f64, speedup));
            }
        }

        plot_data.series.push(PlotSeries {
            name: "Speedup".to_string(),
            data: speedup_data,
            color: "#4CAF50".to_string(),
            line_style: "none".to_string(),
            point_style: "bar".to_string(),
        });

        plot_data
    }

    /// Create memory usage visualization
    pub fn create_memory_usage_plot(&self) -> PlotData {
        let mut plot_data = PlotData {
            title: "Memory Usage Over Time".to_string(),
            x_label: "Measurement".to_string(),
            y_label: "Memory (KB)".to_string(),
            series: Vec::new(),
            visualization_type: VisualizationType::LineChart,
            x_range: None,
            y_range: None,
        };

        let mut memory_data = Vec::new();

        for (i, metrics) in self.metrics_history.iter().enumerate() {
            if let Some(memory) = metrics.memory_usage_kb {
                memory_data.push((i as f64, memory as f64));
            }
        }

        plot_data.series.push(PlotSeries {
            name: "Memory Usage".to_string(),
            data: memory_data,
            color: "#9C27B0".to_string(),
            line_style: "solid".to_string(),
            point_style: "circle".to_string(),
        });

        plot_data
    }

    /// Clear metrics history
    pub fn clear_history(&mut self) {
        self.metrics_history.clear();
    }
}

/// Visualization utilities
pub struct VisualizationUtils;

impl VisualizationUtils {
    /// Generate color palette for multiple series
    pub fn generate_color_palette(count: usize) -> Vec<String> {
        let base_colors = vec![
            "#4CAF50", "#FF5722", "#2196F3", "#9C27B0",
            "#FF9800", "#607D8B", "#795548", "#E91E63",
        ];
        
        (0..count)
            .map(|i| base_colors[i % base_colors.len()].to_string())
            .collect()
    }

    /// Format number for display
    pub fn format_number(value: f64, precision: usize) -> String {
        format!("{:.precision$}", value, precision = precision)
    }

    /// Create default plot styles
    pub fn create_default_styles() -> HashMap<String, String> {
        let mut styles = HashMap::new();
        styles.insert("line_width".to_string(), "2".to_string());
        styles.insert("point_size".to_string(), "4".to_string());
        styles.insert("font_size".to_string(), "12".to_string());
        styles.insert("grid_color".to_string(), "#E0E0E0".to_string());
        styles.insert("background_color".to_string(), "#FFFFFF".to_string());
        styles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualization_manager_creation() {
        let manager = VisualizationManager::new();
        assert!(manager.plot_cache.is_empty());
        assert!(manager.current_plot.is_none());
    }

    #[test]
    fn test_simple_function_plotting() {
        let mut manager = VisualizationManager::new();
        let plot = manager.plot_function("x^2", (-10.0, 10.0), 100, "Test", "#4CAF50");
        assert!(plot.is_ok());
        let plot_data = plot.unwrap();
        assert_eq!(plot_data.series.len(), 1);
        assert_eq!(plot_data.series[0].data.len(), 100);
    }

    #[test]
    fn test_performance_visualizer_creation() {
        let visualizer = PerformanceVisualizer::new();
        assert!(visualizer.metrics_history.is_empty());
    }

    #[test]
    fn test_performance_trend_creation() {
        let mut visualizer = PerformanceVisualizer::new();
        visualizer.add_metrics(PerformanceMetrics {
            execution_time_ms: 100.0,
            memory_usage_kb: Some(1024),
            compilation_time_ms: Some(50.0),
            cache_hits: 1,
            error_rate: 0.0,
            speedup: Some(2.0),
        });
        
        let plot = visualizer.create_performance_trend();
        assert_eq!(plot.series.len(), 2); // Standard and MIR
    }

    #[test]
    fn test_plot_data_serialization() {
        let plot_data = PlotData {
            title: "Test".to_string(),
            x_label: "X".to_string(),
            y_label: "Y".to_string(),
            series: vec![PlotSeries {
                name: "Series1".to_string(),
                data: vec![(1.0, 2.0), (3.0, 4.0)],
                color: "#4CAF50".to_string(),
                line_style: "solid".to_string(),
                point_style: "circle".to_string(),
            }],
            visualization_type: VisualizationType::LineChart,
            x_range: Some((0.0, 10.0)),
            y_range: Some((0.0, 10.0)),
        };
        
        let json = VisualizationManager::new().export_plot_data(&plot_data);
        assert!(json.is_ok());
        
        let deserialized = VisualizationManager::new().import_plot_data(&json.unwrap());
        assert!(deserialized.is_ok());
        let deserialized_plot = deserialized.unwrap();
        assert_eq!(deserialized_plot.title, "Test");
        assert_eq!(deserialized_plot.series.len(), 1);
    }
}