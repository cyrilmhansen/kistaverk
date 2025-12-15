#ifndef NATIVE_BRIDGE_H
#define NATIVE_BRIDGE_H

#include <jni.h>
#include <string>

// Function analysis
extern "C" JNIEXPORT jstring JNICALL
Java_com_kistaverk_FunctionAnalysisViewModel_nativeAnalyzeFunction(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jint iterations,
    jint mode
);

// Automatic differentiation
extern "C" JNIEXPORT jstring JNICALL
Java_com_kistaverk_MathToolViewModel_nativeComputeDerivative(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jstring variable,
    jboolean forwardMode
);

// Visualization
extern "C" JNIEXPORT jstring JNICALL
Java_com_kistaverk_VisualizationViewModel_nativeCreatePlot(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jdouble xMin,
    jdouble xMax,
    jint resolution
);

// Performance analysis
extern "C" JNIEXPORT jstring JNICALL
Java_com_kistaverk_PerformanceAnalyzer_nativeBenchmarkFunction(
    JNIEnv* env,
    jobject thiz,
    jstring expression,
    jint iterations
);

#endif // NATIVE_BRIDGE_H