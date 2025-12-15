# UI Implementation Guide for MIR Advanced Features

## 🎯 Overview

This guide provides implementation instructions for integrating MIR advanced features into kistaverk's Android UI.

## 📋 Phase 1: Function Analysis Screen

### 1.1 Create FunctionAnalysisActivity.kt
```kotlin
class FunctionAnalysisActivity : AppCompatActivity() {
    private lateinit var binding: ActivityFunctionAnalysisBinding
    private lateinit var viewModel: FunctionAnalysisViewModel
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityFunctionAnalysisBinding.inflate(layoutInflater)
        setContentView(binding.root)
        viewModel = ViewModelProvider(this).get(FunctionAnalysisViewModel::class.java)
        setupObservers()
        setupEventListeners()
        setupToolbar()
    }
    
    private fun setupObservers() {
        viewModel.analysisResult.observe(this) { updateAnalysisResult(it) }
        viewModel.plotData.observe(this) { updatePlotView(it) }
        viewModel.errorMessage.observe(this) { showError(it) }
        viewModel.isLoading.observe(this) { updateLoadingState(it) }
    }
    
    private fun performAnalysis() {
        val expression = binding.functionInput.text.toString()
        val iterations = binding.iterationsInput.text.toString().toIntOrNull() ?: 100
        viewModel.analyzeFunction(expression, iterations)
    }
}
```

### 1.2 Create activity_function_analysis.xml
```xml
<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:app="http://schemas.android.com/apk/res-auto"
    android:layout_width="match_parent"
    android:layout_height="match_parent"
    android:orientation="vertical">
    
    <com.google.android.material.appbar.MaterialToolbar
        android:id="@+id/toolbar"/>
    
    <com.google.android.material.textfield.TextInputLayout>
        <com.google.android.material.textfield.TextInputEditText
            android:id="@+id/functionInput"/>
    </com.google.android.material.textfield.TextInputLayout>
    
    <com.google.android.material.chip.ChipGroup
        android:id="@+id/evaluationModeGroup">
        <com.google.android.material.chip.Chip android:text="Standard"/>
        <com.google.android.material.chip.Chip android:text="MIR JIT"/>
    </com.google.android.material.chip.ChipGroup>
    
    <com.kistaverk.visualization.PlotView
        android:id="@+id/plotView"/>
    
    <com.google.android.material.button.MaterialButton
        android:id="@+id/analyzeButton"
        android:text="Analyze"/>
</LinearLayout>
```

### 1.3 Create FunctionAnalysisViewModel.kt
```kotlin
class FunctionAnalysisViewModel : ViewModel() {
    private val _analysisResult = MutableLiveData<FunctionAnalysisResult>()
    val analysisResult: LiveData<FunctionAnalysisResult> = _analysisResult
    
    fun analyzeFunction(expression: String, iterations: Int) {
        viewModelScope.launch {
            try {
                val result = withContext(Dispatchers.IO) {
                    nativeAnalyzeFunction(expression, iterations)
                }
                _analysisResult.value = parseAnalysisResult(result)
            } catch (e: Exception) {
                // Handle error
            }
        }
    }
    
    private external fun nativeAnalyzeFunction(expression: String, iterations: Int): String
}
```

## 📋 Phase 2: Automatic Differentiation UI

### 2.1 Extend MathToolActivity.kt
```kotlin
// Add to MathToolActivity
private fun setupADControls() {
    binding.computeDerivativeButton.setOnClickListener { computeDerivative() }
}

private fun computeDerivative() {
    val expression = binding.expressionInput.text.toString()
    viewModel.computeDerivative(expression, "x", true)
}
```

### 2.2 Extend math_tool_layout.xml
```xml
<LinearLayout android:id="@+id/adSection">
    <com.google.android.material.chip.ChipGroup
        android:id="@+id/adModeGroup">
        <com.google.android.material.chip.Chip android:text="Forward Mode"/>
        <com.google.android.material.chip.Chip android:text="Reverse Mode"/>
    </com.google.android.chip.ChipGroup>
    
    <com.google.android.material.button.MaterialButton
        android:id="@+id/computeDerivativeButton"
        android:text="Compute Derivative"/>
    
    <com.kistaverk.visualization.PlotView
        android:id="@+id/derivativePlot"/>
</LinearLayout>
```

## 📋 Phase 3: Advanced Visualization UI

### 3.1 Create VisualizationActivity.kt
```kotlin
class VisualizationActivity : AppCompatActivity() {
    private lateinit var binding: ActivityVisualizationBinding
    private lateinit var viewModel: VisualizationViewModel
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityVisualizationBinding.inflate(layoutInflater)
        setContentView(binding.root)
        viewModel = ViewModelProvider(this).get(VisualizationViewModel::class.java)
        setupObservers()
    }
    
    private fun createPlot() {
        val expression = binding.plotExpression.text.toString()
        val xMin = binding.xMinInput.text.toString().toDoubleOrNull() ?: -10.0
        val xMax = binding.xMaxInput.text.toString().toDoubleOrNull() ?: 10.0
        val resolution = binding.resolutionInput.text.toString().toIntOrNull() ?: 100
        viewModel.createPlot(expression, xMin, xMax, resolution)
    }
}
```

### 3.2 Create PlotView.kt
```kotlin
class PlotView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null
) : View(context, attrs) {
    
    private var plotData: PlotData? = null
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val gestureDetector: GestureDetector
    private val scaleGestureDetector: ScaleGestureDetector
    
    fun setPlotData(data: PlotData) {
        plotData = data
        invalidate()
    }
    
    override fun onDraw(canvas: Canvas) {
        drawAxes(canvas)
        drawGrid(canvas)
        plotData?.series?.forEach { drawSeries(canvas, it) }
        drawLabels(canvas)
    }
    
    private fun drawLineSeries(canvas: Canvas, series: PlotSeries) {
        // Implement line series drawing
    }
    
    override fun onTouchEvent(event: MotionEvent): Boolean {
        return gestureDetector.onTouchEvent(event)
    }
}
```

## 📋 Phase 4: JNI Bridge Implementation

### 4.1 Create native-bridge.h
```cpp
#ifndef NATIVE_BRIDGE_H
#define NATIVE_BRIDGE_H

#include <jni.h>

extern "C" JNIEXPORT jstring JNICALL
Java_com_kistaverk_FunctionAnalysisViewModel_nativeAnalyzeFunction(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jint iterations
);

extern "C" JNIEXPORT jstring JNICALL
Java_com_kistaverk_MathToolViewModel_nativeComputeDerivative(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jstring variable,
    jboolean forwardMode
);

extern "C" JNIEXPORT jstring JNICALL
Java_com_kistaverk_VisualizationViewModel_nativeCreatePlot(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jdouble xMin,
    jdouble xMax,
    jint resolution
);

#endif
```

### 4.2 Implement native-bridge.cpp
```cpp
#include "native-bridge.h"

// Rust FFI
extern "C" {
    const char* analyze_function(const char* expression, int iterations);
    const char* compute_derivative(const char* expression, const char* variable, bool forwardMode);
    const char* create_plot(const char* expression, double x_min, double x_max, int resolution);
    void free_string(const char* str);
}

JNIEXPORT jstring JNICALL
Java_com_kistaverk_FunctionAnalysisViewModel_nativeAnalyzeFunction(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jint iterations
) {
    const char* expr_chars = env->GetStringUTFChars(expression, nullptr);
    const char* result = analyze_function(expr_chars, iterations);
    env->ReleaseStringUTFChars(expression, expr_chars);
    jstring j_result = env->NewStringUTF(result);
    free_string(result);
    return j_result;
}
```

## 🏁 Conclusion

This guide provides comprehensive instructions for UI implementation. The next steps are:

1. **Implement Android components** following this guide
2. **Extend JNI bridge** to connect Rust backend
3. **Test thoroughly** with unit and integration tests
4. **Optimize performance** for smooth user experience

**Status**: ✅ **GUIDE COMPLETE**
**Date**: 2025-12-15