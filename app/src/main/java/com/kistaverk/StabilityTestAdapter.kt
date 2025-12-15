package com.kistaverk

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView

/**
 * Adapter for displaying stability test results in a RecyclerView
 */
class StabilityTestAdapter(private val testResults: List<StabilityTestResult>) : 
    RecyclerView.Adapter<StabilityTestAdapter.StabilityTestViewHolder>() {
    
    /**
     * ViewHolder for stability test items
     */
    class StabilityTestViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
        val testName: TextView = itemView.findViewById(R.id.testName)
        val testStatus: TextView = itemView.findViewById(R.id.testStatus)
        val testTime: TextView = itemView.findViewById(R.id.testTime)
        val testError: TextView = itemView.findViewById(R.id.testError)
    }
    
    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): StabilityTestViewHolder {
        val view = LayoutInflater.from(parent.context)
            .inflate(R.layout.item_stability_test, parent, false)
        return StabilityTestViewHolder(view)
    }
    
    override fun onBindViewHolder(holder: StabilityTestViewHolder, position: Int) {
        val testResult = testResults[position]
        
        // Set test name
        holder.testName.text = testResult.testName
        
        // Set status (passed/failed)
        val statusText = if (testResult.passed) {
            "✓ Passed"
        } else {
            "✗ Failed"
        }
        holder.testStatus.text = statusText
        holder.testStatus.setTextColor(
            if (testResult.passed) {
                holder.itemView.context.getColor(R.color.success)
            } else {
                holder.itemView.context.getColor(R.color.error)
            }
        )
        
        // Set execution time
        holder.testTime.text = "${testResult.executionTimeMs} ms"
        
        // Set error message if present
        if (testResult.errorMessage != null) {
            holder.testError.text = testResult.errorMessage
            holder.testError.visibility = View.VISIBLE
        } else {
            holder.testError.visibility = View.GONE
        }
    }
    
    override fun getItemCount(): Int = testResults.size
}