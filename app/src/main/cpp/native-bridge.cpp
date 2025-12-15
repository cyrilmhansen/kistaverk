#include "native-bridge.h"
#include <string>
#include <vector>
#include <memory>

// Rust FFI headers
extern "C" {
    // Function analysis
    const char* analyze_function(const char* expression, int iterations, int mode);
    void free_string(const char* str);
    
    // Automatic differentiation
    const char* compute_derivative(const char* expression, const char* variable, bool forward_mode);
    
    // Visualization
    const char* create_plot(const char* expression, double x_min, double x_max, int resolution);
    
    // Performance analysis
    const char* benchmark_function(const char* expression, int iterations);
}

JNIEXPORT jstring JNICALL
Java_com_kistaverk_FunctionAnalysisViewModel_nativeAnalyzeFunction(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jint iterations,
    jint mode
) {
    // Convert Java string to C string
    const char* expr_chars = env->GetStringUTFChars(expression, nullptr);
    if (!expr_chars) {
        return nullptr;
    }
    
    // Call Rust function
    const char* result = analyze_function(expr_chars, iterations, mode);
    
    // Release Java string
    env->ReleaseStringUTFChars(expression, expr_chars);
    
    // Convert result to Java string
    jstring j_result = env->NewStringUTF(result);
    
    // Free Rust memory
    free_string(result);
    
    return j_result;
}

JNIEXPORT jstring JNICALL
Java_com_kistaverk_MathToolViewModel_nativeComputeDerivative(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jstring variable,
    jboolean forwardMode
) {
    // Convert Java strings to C strings
    const char* expr_chars = env->GetStringUTFChars(expression, nullptr);
    const char* var_chars = env->GetStringUTFChars(variable, nullptr);
    
    if (!expr_chars || !var_chars) {
        if (expr_chars) env->ReleaseStringUTFChars(expression, expr_chars);
        if (var_chars) env->ReleaseStringUTFChars(variable, var_chars);
        return nullptr;
    }
    
    // Call Rust function
    const char* result = compute_derivative(expr_chars, var_chars, forwardMode);
    
    // Release Java strings
    env->ReleaseStringUTFChars(expression, expr_chars);
    env->ReleaseStringUTFChars(variable, var_chars);
    
    // Convert result to Java string
    jstring j_result = env->NewStringUTF(result);
    
    // Free Rust memory
    free_string(result);
    
    return j_result;
}

JNIEXPORT jstring JNICALL
Java_com_kistaverk_VisualizationViewModel_nativeCreatePlot(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jdouble xMin,
    jdouble xMax,
    jint resolution
) {
    // Convert Java string to C string
    const char* expr_chars = env->GetStringUTFChars(expression, nullptr);
    if (!expr_chars) {
        return nullptr;
    }
    
    // Call Rust function
    const char* result = create_plot(expr_chars, xMin, xMax, resolution);
    
    // Release Java string
    env->ReleaseStringUTFChars(expression, expr_chars);
    
    // Convert result to Java string
    jstring j_result = env->NewStringUTF(result);
    
    // Free Rust memory
    free_string(result);
    
    return j_result;
}

JNIEXPORT jstring JNICALL
Java_com_kistaverk_PerformanceAnalyzer_nativeBenchmarkFunction(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jint iterations
) {
    // Convert Java string to C string
    const char* expr_chars = env->GetStringUTFChars(expression, nullptr);
    if (!expr_chars) {
        return nullptr;
    }
    
    // Call Rust function
    const char* result = benchmark_function(expr_chars, iterations);
    
    // Release Java string
    env->ReleaseStringUTFChars(expression, expr_chars);
    
    // Convert result to Java string
    jstring j_result = env->NewStringUTF(result);
    
    // Free Rust memory
    free_string(result);
    
    return j_result;
}