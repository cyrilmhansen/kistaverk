package com.kistaverk.visualization

import android.content.Context
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Path
import android.graphics.Rect
import android.util.AttributeSet
import android.view.GestureDetector
import android.view.MotionEvent
import android.view.ScaleGestureDetector
import android.view.View
import androidx.core.content.ContextCompat
import com.kistaverk.PlotData
import com.kistaverk.PlotSeries
import com.kistaverk.VisualizationType

/**
 * Custom view for displaying plots and charts
 * Supports multiple visualization types and interactive features
 */
class PlotView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = 0
) : View(context, attrs, defStyleAttr), 
    GestureDetector.OnGestureListener, 
    ScaleGestureDetector.OnScaleGestureListener {
    
    // Plot data
    private var plotData: PlotData? = null
    
    // Paints for drawing
    private val axisPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val gridPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val textPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val seriesPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    
    // Gesture detectors
    private val gestureDetector: GestureDetector
    private val scaleGestureDetector: ScaleGestureDetector
    
    // View state
    private var scaleFactor = 1.0f
    private var offsetX = 0.0f
    private var offsetY = 0.0f
    private var minScale = 0.5f
    private var maxScale = 4.0f
    
    // Dimensions
    private var contentWidth = 0f
    private var contentHeight = 0f
    private var plotAreaLeft = 0f
    private var plotAreaTop = 0f
    private var plotAreaRight = 0f
    private var plotAreaBottom = 0f
    
    // Data bounds
    private var dataXMin = 0.0
    private var dataXMax = 1.0
    private var dataYMin = 0.0
    private var dataYMax = 1.0
    
    init {
        // Initialize paints
        axisPaint.apply {
            color = ContextCompat.getColor(context, android.R.color.black)
            strokeWidth = 2f
            style = Paint.Style.STROKE
        }
        
        gridPaint.apply {
            color = ContextCompat.getColor(context, android.R.color.darker_gray)
            strokeWidth = 1f
            style = Paint.Style.STROKE
            alpha = 128
        }
        
        textPaint.apply {
            color = ContextCompat.getColor(context, android.R.color.black)
            textSize = 32f
            textAlign = Paint.Align.CENTER
        }
        
        seriesPaint.apply {
            strokeWidth = 3f
            style = Paint.Style.STROKE
        }
        
        // Initialize gesture detectors
        gestureDetector = GestureDetector(context, this)
        scaleGestureDetector = ScaleGestureDetector(context, this)
    }
    
    /**
     * Set plot data to display
     * @param data PlotData object containing series and configuration
     */
    fun setPlotData(data: PlotData) {
        plotData = data
        
        // Calculate data bounds
        calculateDataBounds()
        
        // Reset view
        resetView()
        
        // Redraw
        invalidate()
    }
    
    /**
     * Calculate data bounds from plot data
     */
    private fun calculateDataBounds() {
        plotData?.let { data ->
            // Calculate X bounds
            val xValues = data.series.flatMap { it.data.map { point -> point.first } }
            if (xValues.isNotEmpty()) {
                dataXMin = xValues.minOrNull() ?: 0.0
                dataXMax = xValues.maxOrNull() ?: 1.0
                
                // Add 10% padding
                val xRange = dataXMax - dataXMin
                dataXMin -= xRange * 0.1
                dataXMax += xRange * 0.1
            }
            
            // Calculate Y bounds
            val yValues = data.series.flatMap { it.data.map { point -> point.second } }
            if (yValues.isNotEmpty()) {
                dataYMin = yValues.minOrNull() ?: 0.0
                dataYMax = yValues.maxOrNull() ?: 1.0
                
                // Add 10% padding
                val yRange = dataYMax - dataYMin
                dataYMin -= yRange * 0.1
                dataYMax += yRange * 0.1
            }
        }
    }
    
    /**
     * Reset view to default state
     */
    private fun resetView() {
        scaleFactor = 1.0f
        offsetX = 0.0f
        offsetY = 0.0f
    }
    
    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        contentWidth = w.toFloat()
        contentHeight = h.toFloat()
        
        // Calculate plot area (leave space for labels)
        plotAreaLeft = 60f
        plotAreaTop = 40f
        plotAreaRight = contentWidth - 20f
        plotAreaBottom = contentHeight - 40f
    }
    
    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        
        if (plotData == null) {
            drawPlaceholder(canvas)
            return
        }
        
        // Draw background
        canvas.drawColor(ContextCompat.getColor(context, android.R.color.white))
        
        // Draw axes and grid
        drawAxes(canvas)
        drawGrid(canvas)
        
        // Draw plot series
        plotData?.series?.forEach { series ->
            drawSeries(canvas, series)
        }
        
        // Draw labels
        drawLabels(canvas)
    }
    
    /**
     * Draw placeholder when no data is available
     */
    private fun drawPlaceholder(canvas: Canvas) {
        canvas.drawColor(ContextCompat.getColor(context, android.R.color.white))
        
        textPaint.apply {
            textAlign = Paint.Align.CENTER
            textSize = 48f
            color = ContextCompat.getColor(context, android.R.color.darker_gray)
        }
        
        canvas.drawText(
            "No plot data available",
            contentWidth / 2,
            contentHeight / 2,
            textPaint
        )
    }
    
    /**
     * Draw X and Y axes
     */
    private fun drawAxes(canvas: Canvas) {
        // Draw X axis
        canvas.drawLine(
            plotAreaLeft, plotAreaBottom,
            plotAreaRight, plotAreaBottom,
            axisPaint
        )
        
        // Draw Y axis
        canvas.drawLine(
            plotAreaLeft, plotAreaTop,
            plotAreaLeft, plotAreaBottom,
            axisPaint
        )
    }
    
    /**
     * Draw grid lines
     */
    private fun drawGrid(canvas: Canvas) {
        // Draw horizontal grid lines
        val yStep = (plotAreaBottom - plotAreaTop) / 10
        for (i in 1..9) {
            val y = plotAreaBottom - i * yStep
            canvas.drawLine(plotAreaLeft, y, plotAreaRight, y, gridPaint)
        }
        
        // Draw vertical grid lines
        val xStep = (plotAreaRight - plotAreaLeft) / 10
        for (i in 1..9) {
            val x = plotAreaLeft + i * xStep
            canvas.drawLine(x, plotAreaTop, x, plotAreaBottom, gridPaint)
        }
    }
    
    /**
     * Draw plot series based on visualization type
     */
    private fun drawSeries(canvas: Canvas, series: PlotSeries) {
        when (plotData?.visualizationType) {
            VisualizationType.LINE_CHART -> drawLineSeries(canvas, series)
            VisualizationType.BAR_CHART -> drawBarSeries(canvas, series)
            VisualizationType.SCATTER_PLOT -> drawScatterSeries(canvas, series)
            VisualizationType.HEAT_MAP -> drawHeatMapSeries(canvas, series)
            VisualizationType.SURFACE_PLOT -> drawSurfaceSeries(canvas, series)
            null -> {}
        }
    }
    
    /**
     * Draw line series
     */
    private fun drawLineSeries(canvas: Canvas, series: PlotSeries) {
        // Set paint color
        seriesPaint.color = android.graphics.Color.parseColor(series.color)
        seriesPaint.style = Paint.Style.STROKE
        
        // Create path
        val path = Path()
        
        // Get data points
        val data = series.data
        if (data.isEmpty()) return
        
        // Map first point
        val firstX = mapX(data[0].first)
        val firstY = mapY(data[0].second)
        path.moveTo(firstX, firstY)
        
        // Map remaining points
        for (i in 1 until data.size) {
            val x = mapX(data[i].first)
            val y = mapY(data[i].second)
            path.lineTo(x, y)
        }
        
        // Draw path
        canvas.drawPath(path, seriesPaint)
    }
    
    /**
     * Draw bar series
     */
    private fun drawBarSeries(canvas: Canvas, series: PlotSeries) {
        // Set paint color
        seriesPaint.color = android.graphics.Color.parseColor(series.color)
        seriesPaint.style = Paint.Style.FILL
        
        // Get data points
        val data = series.data
        if (data.isEmpty()) return
        
        // Calculate bar width
        val barWidth = (plotAreaRight - plotAreaLeft) / (data.size * 1.5f)
        
        // Draw bars
        for ((i, point) in data.withIndex()) {
            val x = mapX(i.toDouble())
            val y = mapY(point.second)
            
            val left = x - barWidth / 2
            val right = x + barWidth / 2
            val top = y
            val bottom = plotAreaBottom
            
            canvas.drawRect(left, top, right, bottom, seriesPaint)
        }
    }
    
    /**
     * Draw scatter series
     */
    private fun drawScatterSeries(canvas: Canvas, series: PlotSeries) {
        // Set paint color
        seriesPaint.color = android.graphics.Color.parseColor(series.color)
        seriesPaint.style = Paint.Style.FILL
        
        // Get data points
        val data = series.data
        if (data.isEmpty()) return
        
        // Draw points
        for (point in data) {
            val x = mapX(point.first)
            val y = mapY(point.second)
            
            canvas.drawCircle(x, y, 8f, seriesPaint)
        }
    }
    
    /**
     * Draw heat map series (placeholder)
     */
    private fun drawHeatMapSeries(canvas: Canvas, series: PlotSeries) {
        // Implementation for heat map would go here
        // For now, draw as scatter plot
        drawScatterSeries(canvas, series)
    }
    
    /**
     * Draw surface series (placeholder)
     */
    private fun drawSurfaceSeries(canvas: Canvas, series: PlotSeries) {
        // Implementation for surface plot would go here
        // For now, draw as line series
        drawLineSeries(canvas, series)
    }
    
    /**
     * Draw labels and titles
     */
    private fun drawLabels(canvas: Canvas) {
        // Draw X axis label
        textPaint.apply {
            textAlign = Paint.Align.CENTER
            textSize = 32f
            color = ContextCompat.getColor(context, android.R.color.black)
        }
        
        canvas.drawText(
            plotData?.xLabel ?: "X",
            (plotAreaLeft + plotAreaRight) / 2,
            plotAreaBottom + 30f,
            textPaint
        )
        
        // Draw Y axis label
        textPaint.textAlign = Paint.Align.CENTER
        canvas.save()
        canvas.rotate(-90f, plotAreaLeft - 30f, (plotAreaTop + plotAreaBottom) / 2)
        canvas.drawText(
            plotData?.yLabel ?: "Y",
            plotAreaLeft - 30f,
            (plotAreaTop + plotAreaBottom) / 2,
            textPaint
        )
        canvas.restore()
        
        // Draw title
        textPaint.textAlign = Paint.Align.CENTER
        textPaint.textSize = 36f
        canvas.drawText(
            plotData?.title ?: "Plot",
            (plotAreaLeft + plotAreaRight) / 2,
            plotAreaTop - 10f,
            textPaint
        )
    }
    
    /**
     * Map X coordinate from data space to screen space
     */
    private fun mapX(value: Double): Float {
        val normalized = ((value - dataXMin) / (dataXMax - dataXMin)).toFloat()
        return plotAreaLeft + normalized * (plotAreaRight - plotAreaLeft)
    }
    
    /**
     * Map Y coordinate from data space to screen space
     */
    private fun mapY(value: Double): Float {
        val normalized = 1f - ((value - dataYMin) / (dataXMax - dataXMin)).toFloat()
        return plotAreaTop + normalized * (plotAreaBottom - plotAreaTop)
    }
    
    // Gesture handling for zoom and pan
    override fun onDown(e: MotionEvent): Boolean = true
    
    override fun onShowPress(e: MotionEvent) {}
    
    override fun onSingleTapUp(e: MotionEvent): Boolean = true
    
    override fun onScroll(e1: MotionEvent?, e2: MotionEvent, distanceX: Float, distanceY: Float): Boolean {
        offsetX -= distanceX
        offsetY -= distanceY
        invalidate()
        return true
    }
    
    override fun onLongPress(e: MotionEvent) {}
    
    override fun onFling(e1: MotionEvent?, e2: MotionEvent, velocityX: Float, velocityY: Float): Boolean = false
    
    override fun onScale(detector: ScaleGestureDetector): Boolean {
        scaleFactor *= detector.scaleFactor
        scaleFactor = scaleFactor.coerceIn(minScale, maxScale)
        invalidate()
        return true
    }
    
    override fun onScaleBegin(detector: ScaleGestureDetector): Boolean = true
    
    override fun onScaleEnd(detector: ScaleGestureDetector) {}
    
    override fun onTouchEvent(event: MotionEvent): Boolean {
        scaleGestureDetector.onTouchEvent(event)
        return gestureDetector.onTouchEvent(event)
    }
}