package com.kistaverk

import android.os.Bundle
import android.view.View
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.ViewModelProvider
import com.kistaverk.databinding.ActivityFunctionAnalysisBinding

/**
 * Activity for function analysis screen
 * Provides performance comparison and stability testing for mathematical functions
 */
class FunctionAnalysisActivity : AppCompatActivity() {
    
    private lateinit var binding: ActivityFunctionAnalysisBinding
    private lateinit var viewModel: FunctionAnalysisViewModel
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Initialize view binding
        binding = ActivityFunctionAnalysisBinding.inflate(layoutInflater)
        setContentView(binding.root)
        
        // Initialize ViewModel
        viewModel = ViewModelProvider(this).get(FunctionAnalysisViewModel::class.java)
        
        // Set up observers for LiveData
        setupObservers()
        
        // Set up event listeners
        setupEventListeners()
        
        // Set up toolbar
        setupToolbar()
    }
    
    private fun setupObservers() {
        // Observe analysis results
        viewModel.analysisResult.observe(this) { result ->
            updateAnalysisResult(result)
        }
        
        // Observe plot data
        viewModel.plotData.observe(this) { plotData ->
            updatePlotView(plotData)
        }
        
        // Observe error messages
        viewModel.errorMessage.observe(this) { error ->
            showError(error)
        }
        
        // Observe loading state
        viewModel.isLoading.observe(this) { isLoading ->
            updateLoadingState(isLoading)
        }
    }
    
    private fun setupEventListeners() {
        // Analyze button click
        binding.analyzeButton.setOnClickListener {
            performAnalysis()
        }
        
        // Export button click
        binding.exportButton.setOnClickListener {
            exportResults()
        }
        
        // Evaluation mode selection
        binding.evaluationModeGroup.setOnCheckedChangeListener { group, checkedId ->
            updateEvaluationMode(checkedId)
        }
    }
    
    private fun performAnalysis() {
        // Get input values
        val expression = binding.functionInput.text.toString()
        val iterations = binding.iterationsInput.text.toString().toIntOrNull() ?: 100
        
        // Validate input
        if (expression.isBlank()) {
            showError("Please enter a function expression")
            return
        }
        
        // Start analysis
        viewModel.analyzeFunction(expression, iterations)
    }
    
    private fun exportResults() {
        // Get current plot data
        val plotData = viewModel.plotData.value
        if (plotData != null) {
            viewModel.exportPlotData(plotData)
        } else {
            showError("No analysis results to export")
        }
    }
    
    private fun updateAnalysisResult(result: FunctionAnalysisResult) {
        // Update performance metrics
        binding.executionTime.text = "${result.standardMetrics.executionTimeMs} ms"
        binding.memoryUsage.text = "${result.standardMetrics.memoryUsageKb ?: 0} KB"
        binding.speedupFactor.text = "${result.standardMetrics.speedup?.toString() ?: "N/A"}"
        
        // Update recommendation
        binding.recommendationText.text = result.recommendation
        
        // Update stability test results
        updateStabilityResults(result.stabilityResults)
    }
    
    private fun updateStabilityResults(results: List<StabilityTestResult>) {
        // Create adapter for stability results
        val adapter = StabilityTestAdapter(results)
        binding.stabilityResults.adapter = adapter
    }
    
    private fun updatePlotView(plotData: PlotData) {
        binding.plotView.setPlotData(plotData)
        binding.plotView.invalidate()
    }
    
    private fun showError(error: String?) {
        error?.let {
            // Show error message
            binding.errorText.text = it
            binding.errorText.visibility = View.VISIBLE
        } ?: run {
            binding.errorText.visibility = View.GONE
        }
    }
    
    private fun updateLoadingState(isLoading: Boolean) {
        if (isLoading) {
            binding.loadingIndicator.visibility = View.VISIBLE
            binding.analyzeButton.isEnabled = false
        } else {
            binding.loadingIndicator.visibility = View.GONE
            binding.analyzeButton.isEnabled = true
        }
    }
    
    private fun updateEvaluationMode(checkedId: Int) {
        val mode = when (checkedId) {
            R.id.standardChip -> EvaluationMode.STANDARD
            R.id.mirChip -> EvaluationMode.MIR
            R.id.hybridChip -> EvaluationMode.HYBRID
            else -> EvaluationMode.STANDARD
        }
        viewModel.setEvaluationMode(mode)
    }
    
    private fun setupToolbar() {
        setSupportActionBar(binding.toolbar)
        supportActionBar?.setDisplayHomeAsUpEnabled(true)
        supportActionBar?.setDisplayShowHomeEnabled(true)
    }
    
    override fun onSupportNavigateUp(): Boolean {
        onBackPressed()
        return true
    }
}

/**
 * Evaluation modes for function analysis
 */
enum class EvaluationMode {
    STANDARD,
    MIR,
    HYBRID
}