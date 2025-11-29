package aeska.kistaverk

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat

class SensorLoggerService : Service() {
    companion object {
        private const val CHANNEL_ID = "sensor_logger_channel"
        private const val CHANNEL_NAME = "Sensor Logger"
        private const val NOTIFICATION_ID = 341
        private const val ACTION_START = "aeska.kistaverk.sensor_logger.START"
        private const val ACTION_STOP = "aeska.kistaverk.sensor_logger.STOP"
        private const val EXTRA_STATUS = "aeska.kistaverk.sensor_logger.status"

        fun start(context: Context, status: String?) {
            val intent = Intent(context, SensorLoggerService::class.java).apply {
                action = ACTION_START
                putExtra(EXTRA_STATUS, status)
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun stop(context: Context) {
            val intent = Intent(context, SensorLoggerService::class.java).apply {
                action = ACTION_STOP
            }
            context.startService(intent)
        }
    }

    private var currentStatus: String? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        ensureChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> {
                currentStatus = intent.getStringExtra(EXTRA_STATUS)
                startForeground(NOTIFICATION_ID, buildNotification(currentStatus))
                return START_STICKY
            }
            ACTION_STOP -> {
                stopForeground(true)
                stopSelf()
                return START_NOT_STICKY
            }
            else -> return START_NOT_STICKY
        }
    }

    private fun buildNotification(status: String?): Notification {
        val openApp = Intent(this, MainActivity::class.java).apply {
            action = "aeska.kistaverk.sensor_logger"
        }
        val pendingIntentFlags = PendingIntent.FLAG_UPDATE_CURRENT or
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) PendingIntent.FLAG_IMMUTABLE else 0
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            openApp,
            pendingIntentFlags
        )
        val text = status?.takeIf { it.isNotBlank() } ?: "Sensor logging running"
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Sensor logger")
            .setContentText(text)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()
    }

    private fun ensureChannel() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val nm = getSystemService(Context.NOTIFICATION_SERVICE) as? NotificationManager ?: return
        if (nm.getNotificationChannel(CHANNEL_ID) != null) return
        val channel = NotificationChannel(
            CHANNEL_ID,
            CHANNEL_NAME,
            NotificationManager.IMPORTANCE_LOW
        )
        nm.createNotificationChannel(channel)
    }
}
