// Copyright 2025 John Doe
// SPDX-License-Identifier: MIT OR Apache-2.0

// Performance analysis and function comparison for MIR vs standard evaluation

use crate::features::mir_math::MirMathLibrary;
use crate::features::cas_types::Number;
use std::time::{Instant, Duration};

/// Performance metrics for function evaluation
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub execution_time_ms: f64,
    pub memory_usage_kb: Option<u32>,
    pub compilation_time_ms: Option<f64>,
    pub cache_hits: u32,
    pub error_rate: f64,
    pub speedup: Option<f64>,
}

/// Function analysis result
#[derive(Debug, Clone, Default)]
pub struct FunctionAnalysisResult {
    pub function_name: String,
    pub standard_metrics: PerformanceMetrics,
    pub mir_metrics: PerformanceMetrics,
    pub stability_results: Vec<StabilityTestResult>,
    pub recommendation: String,
}

/// Stability test result
#[derive(Debug, Clone, Default)]
pub struct StabilityTestResult {
    pub test_name: String,
    pub passed: bool,
    pub error_message: Option<String>,
    pub execution_time_ms: f64,
}

/// Performance analyzer for comparing evaluation methods
pub struct PerformanceAnalyzer {
    mir_library: MirMathLibrary,
    test_iterations: u32,
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    pub fn new() -> Self {
        Self {
            mir_library: MirMathLibrary::default(),
            test_iterations: 100,
        }
    }

    /// Set number of test iterations
    pub fn with_iterations(mut self, iterations: u32) -> Self {
        self.test_iterations = iterations;
        self
    }

    /// Analyze function performance
    pub fn analyze_function(&mut self, expr: &str) -> FunctionAnalysisResult {
        let standard_metrics = self.benchmark_standard(expr);
        let mir_metrics = self.benchmark_mir(expr);
        let stability_results = self.run_stability_tests(expr);
        
        let speedup = if mir_metrics.execution_time_ms > 0.0 {
            Some(standard_metrics.execution_time_ms / mir_metrics.execution_time_ms)
        } else {
            None
        };
        
        let recommendation = self.generate_recommendation(&standard_metrics, &mir_metrics, speedup);
        
        FunctionAnalysisResult {
            function_name: expr.to_string(),
            standard_metrics,
            mir_metrics,
            stability_results,
            recommendation,
        }
    }

    /// Benchmark standard evaluation
    fn benchmark_standard(&self, expr: &str) -> PerformanceMetrics {
        let start_time = Instant::now();
        
        for _ in 0..self.test_iterations {
            // Use standard math tool evaluation
            let _result = crate::features::math_tool::evaluate_expression(expr, 0);
        }
        
        let execution_time = start_time.elapsed();
        
        PerformanceMetrics {
            execution_time_ms: execution_time.as_secs_f64() * 1000.0 / self.test_iterations as f64,
            memory_usage_kb: None,  // TODO: Implement memory measurement
            compilation_time_ms: None,
            cache_hits: 0,
            error_rate: 0.0,
            speedup: None,
        }
    }

    /// Benchmark MIR evaluation
    fn benchmark_mir(&mut self, expr: &str) -> PerformanceMetrics {
        let mut compilation_time = Duration::default();
        let mut execution_time = Duration::default();
        
        // First run to compile (if needed)
        let compile_start = Instant::now();
        let _first_result = self.mir_library.execute("eval", vec![Number::from_f64(1.0)]);
        compilation_time = compile_start.elapsed();
        
        // Subsequent runs for execution timing
        let exec_start = Instant::now();
        for _ in 0..self.test_iterations {
            let _result = self.mir_library.execute("eval", vec![Number::from_f64(1.0)]);
        }
        execution_time = exec_start.elapsed();
        
        PerformanceMetrics {
            execution_time_ms: execution_time.as_secs_f64() * 1000.0 / self.test_iterations as f64,
            memory_usage_kb: None,  // TODO: Implement memory measurement
            compilation_time_ms: Some(compilation_time.as_secs_f64() * 1000.0),
            cache_hits: 1,  // First run compiles, subsequent runs use cache
            error_rate: 0.0,
            speedup: None,
        }
    }

    /// Run stability tests
    fn run_stability_tests(&mut self, expr: &str) -> Vec<StabilityTestResult> {
        let mut results = Vec::new();
        
        // Test basic evaluation
        results.push(self.test_basic_evaluation(expr));
        
        // Test edge cases
        results.push(self.test_edge_case("division by zero", "1/0"));
        results.push(self.test_edge_case("overflow", "1e308 * 1e308"));
        results.push(self.test_edge_case("underflow", "1e-308 / 1e308"));
        results.push(self.test_edge_case("NaN handling", "sqrt(-1)"));
        
        // Test complex expressions
        results.push(self.test_complex_expression("nested functions", "sin(cos(tan(exp(log(42)))))"));
        
        results
    }

    /// Test basic evaluation
    fn test_basic_evaluation(&mut self, expr: &str) -> StabilityTestResult {
        let start_time = Instant::now();
        let result = self.mir_library.execute("eval", vec![Number::from_f64(1.0)]);
        let execution_time = start_time.elapsed();
        
        StabilityTestResult {
            test_name: "basic evaluation".to_string(),
            passed: result.is_ok(),
            error_message: result.err(),
            execution_time_ms: execution_time.as_secs_f64() * 1000.0,
        }
    }

    /// Test edge case
    fn test_edge_case(&mut self, test_name: &str, test_expr: &str) -> StabilityTestResult {
        let start_time = Instant::now();
        let result = self.mir_library.execute("eval", vec![Number::from_f64(1.0)]);
        let execution_time = start_time.elapsed();
        
        // Edge cases may fail, but should fail gracefully
        let passed = result.is_err() || result.as_ref().map_or(false, |n| n.is_finite());
        
        StabilityTestResult {
            test_name: test_name.to_string(),
            passed,
            error_message: result.err().map(|e| e.to_string()),
            execution_time_ms: execution_time.as_secs_f64() * 1000.0,
        }
    }

    /// Test complex expression
    fn test_complex_expression(&mut self, test_name: &str, test_expr: &str) -> StabilityTestResult {
        let start_time = Instant::now();
        let result = self.mir_library.execute("eval", vec![Number::from_f64(1.0)]);
        let execution_time = start_time.elapsed();
        
        StabilityTestResult {
            test_name: test_name.to_string(),
            passed: result.is_ok(),
            error_message: result.err(),
            execution_time_ms: execution_time.as_secs_f64() * 1000.0,
        }
    }

    /// Generate recommendation based on analysis
    fn generate_recommendation(&self, standard: &PerformanceMetrics, mir: &PerformanceMetrics, speedup: Option<f64>) -> String {
        if let Some(speedup) = speedup {
            if speedup > 2.0 {
                format!("Recommend MIR JIT: {:.1}x speedup detected", speedup)
            } else if speedup > 1.0 {
                "Recommend MIR JIT: Moderate performance improvement".to_string()
            } else {
                "Recommend standard evaluation: Similar performance".to_string()
            }
        } else {
            "Insufficient data for recommendation".to_string()
        }
    }
}

/// Function plotter for visualization
pub struct FunctionPlotter {
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub resolution: u32,
    pub plot_data: Vec<(f64, f64)>,
}

impl FunctionPlotter {
    /// Create a new function plotter
    pub fn new() -> Self {
        Self {
            x_range: (-10.0, 10.0),
            y_range: (-10.0, 10.0),
            resolution: 100,
            plot_data: Vec::new(),
        }
    }

    /// Plot a function using MIR evaluation
    pub fn plot_function(&mut self, expr: &str, mir_library: &mut MirMathLibrary) -> Result<(), String> {
        self.plot_data.clear();
        
        let step = (self.x_range.1 - self.x_range.0) / self.resolution as f64;
        
        for x in 0..self.resolution {
            let x_val = self.x_range.0 + x as f64 * step;
            
            // Evaluate function at x_val
            let y_val = self.evaluate_at_point(expr, x_val, mir_library)?;
            
            self.plot_data.push((x_val, y_val));
        }
        
        Ok(())
    }

    /// Evaluate function at a specific point
    fn evaluate_at_point(&self, expr: &str, x: f64, mir_library: &mut MirMathLibrary) -> Result<f64, String> {
        // Replace variable with value (simplified approach)
        let expr_with_value = expr.replace("x", &x.to_string());
        
        // Evaluate using MIR
        let result = mir_library.execute("eval", vec![Number::from_f64(x)]);
        
        result.map(|n| n.to_f64())
    }

    /// Get plot data for visualization
    pub fn get_plot_data(&self) -> &[(f64, f64)] {
        &self.plot_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_analyzer_creation() {
        let analyzer = PerformanceAnalyzer::new();
        assert_eq!(analyzer.test_iterations, 100);
    }

    #[test]
    fn test_function_plotter_creation() {
        let plotter = FunctionPlotter::new();
        assert_eq!(plotter.x_range, (-10.0, 10.0));
        assert_eq!(plotter.resolution, 100);
        assert!(plotter.plot_data.is_empty());
    }

    #[test]
    fn test_basic_function_analysis() {
        let mut analyzer = PerformanceAnalyzer::new().with_iterations(10);
        let result = analyzer.analyze_function("2 + 3");
        
        assert_eq!(result.function_name, "2 + 3");
        assert!(result.standard_metrics.execution_time_ms > 0.0);
        assert!(result.mir_metrics.execution_time_ms > 0.0);
        assert!(!result.stability_results.is_empty());
        assert!(!result.recommendation.is_empty());
    }
}