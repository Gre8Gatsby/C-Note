package com.cnote.app.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import android.net.Uri
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import android.provider.ContactsContract
import android.util.Log
import androidx.core.app.NotificationCompat
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import okio.ByteString.Companion.toByteString
import org.json.JSONObject
import java.util.concurrent.TimeUnit
import kotlin.concurrent.thread

class CallStreamService : Service() {

    companion object {
        private const val TAG = "CallStreamService"
        private const val NOTIFICATION_CHANNEL_ID = "c_note_call_streaming"
        private const val NOTIFICATION_ID = 1001

        const val ACTION_START_STREAMING = "com.cnote.app.START_STREAMING"
        const val ACTION_STOP_STREAMING = "com.cnote.app.STOP_STREAMING"
        const val EXTRA_PHONE_NUMBER = "extra_phone_number"
        const val EXTRA_TIMESTAMP = "extra_timestamp"

        // Backend WebSocket endpoint (configurable)
        var BACKEND_WS_URL = "ws://10.0.2.2:8080/ws/live-call"
    }

    private var audioRecord: AudioRecord? = null
    private var isRecording = false
    private var webSocket: WebSocket? = null
    private var wakeLock: PowerManager.WakeLock? = null
    private val httpClient = OkHttpClient.Builder()
        .readTimeout(0, TimeUnit.MILLISECONDS)
        .build()

    private var currentContactName: String = "Unknown"
    private var currentPhoneNumber: String = ""

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()

        val powerManager = getSystemService(Context.POWER_SERVICE) as PowerManager
        wakeLock = powerManager.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "CNote::CallAudioStreamingLock")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val action = intent?.action ?: return START_NOT_STICKY

        when (action) {
            ACTION_START_STREAMING -> {
                currentPhoneNumber = intent.getStringExtra(EXTRA_PHONE_NUMBER) ?: "Unknown"
                currentContactName = resolveContactName(currentPhoneNumber)

                startForeground(NOTIFICATION_ID, buildForegroundNotification("Call connected: $currentContactName. Streaming live transcription..."))
                wakeLock?.acquire(30 * 60 * 1000L) // 30 mins max timeout

                connectWebSocketAndStreamAudio()
            }

            ACTION_STOP_STREAMING -> {
                stopStreamingAndFinalize()
            }
        }

        return START_NOT_STICKY
    }

    private fun connectWebSocketAndStreamAudio() {
        val request = Request.Builder().url(BACKEND_WS_URL).build()

        webSocket = httpClient.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(ws: WebSocket, response: Response) {
                Log.d(TAG, "Connected to C Note live stream WebSocket")

                // 1. Send handshake control message
                val handshake = JSONObject().apply {
                    put("action", "start")
                    put("contact_name", currentContactName)
                    put("phone_number", currentPhoneNumber)
                }
                ws.send(handshake.toString())

                // 2. Start audio capture loop
                startAudioCapture(ws)
            }

            override fun onMessage(ws: WebSocket, text: String) {
                try {
                    val json = JSONObject(text)
                    val type = json.optString("type")

                    when (type) {
                        "transcript_interim" -> {
                            val interim = json.optString("text")
                            updateNotification("Live: $interim")
                        }
                        "transcript_final" -> {
                            val sentence = json.optString("text")
                            Log.d(TAG, "Final sentence: $sentence")
                        }
                        "live_action_item" -> {
                            val action = json.optString("item")
                            updateNotification("⚡ Commitment: $action")
                        }
                        "call_completed" -> {
                            Log.d(TAG, "Call completed and note stored!")
                            stopSelf()
                        }
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Error parsing incoming WS message: ${e.message}")
                }
            }

            override fun onFailure(ws: WebSocket, t: Throwable, response: Response?) {
                Log.e(TAG, "WebSocket failure: ${t.message}")
            }
        })
    }

    private fun startAudioCapture(ws: WebSocket) {
        val sampleRate = 16000
        val channelConfig = AudioFormat.CHANNEL_IN_MONO
        val audioFormat = AudioFormat.ENCODING_PCM_16BIT
        val minBufferSize = AudioRecord.getMinBufferSize(sampleRate, channelConfig, audioFormat)
        val bufferSize = minBufferSize.coerceAtLeast(3200) // ~100ms buffer

        try {
            // Use VOICE_COMMUNICATION for noise suppression and acoustic echo cancellation
            audioRecord = AudioRecord(
                MediaRecorder.AudioSource.VOICE_COMMUNICATION,
                sampleRate,
                channelConfig,
                audioFormat,
                bufferSize
            )

            if (audioRecord?.state != AudioRecord.STATE_INITIALIZED) {
                // Fallback to MIC if VOICE_COMMUNICATION is restricted by OEM
                audioRecord = AudioRecord(
                    MediaRecorder.AudioSource.MIC,
                    sampleRate,
                    channelConfig,
                    audioFormat,
                    bufferSize
                )
            }

            audioRecord?.startRecording()
            isRecording = true

            thread(name = "CNoteAudioCaptureThread") {
                val buffer = ByteArray(bufferSize)
                while (isRecording && audioRecord?.recordingState == AudioRecord.RECORDSTATE_RECORDING) {
                    val readBytes = audioRecord?.read(buffer, 0, buffer.size) ?: -1
                    if (readBytes > 0) {
                        val chunk = buffer.copyOf(readBytes)
                        ws.send(chunk.toByteString())
                    }
                }
            }
        } catch (e: SecurityException) {
            Log.e(TAG, "Permission denied for audio recording: ${e.message}")
            stopSelf()
        } catch (e: Exception) {
            Log.e(TAG, "Error starting AudioRecord: ${e.message}")
            stopSelf()
        }
    }

    private fun stopStreamingAndFinalize() {
        isRecording = false
        try {
            audioRecord?.stop()
            audioRecord?.release()
            audioRecord = null
        } catch (e: Exception) {
            Log.e(TAG, "Error stopping audioRecord: ${e.message}")
        }

        // Send finish signal to server
        val finishMsg = JSONObject().apply {
            put("action", "finish")
        }
        webSocket?.send(finishMsg.toString())

        if (wakeLock?.isHeld == true) {
            wakeLock?.release()
        }
    }

    private fun resolveContactName(phoneNumber: String): String {
        if (phoneNumber.isEmpty() || phoneNumber == "Unknown") return "Unknown"
        val uri = Uri.withAppendedPath(ContactsContract.PhoneLookup.CONTENT_FILTER_URI, Uri.encode(phoneNumber))
        val projection = arrayOf(ContactsContract.PhoneLookup.DISPLAY_NAME)

        try {
            contentResolver.query(uri, projection, null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    return cursor.getString(0) ?: phoneNumber
                }
            }
        } catch (e: Exception) {
            Log.w(TAG, "Could not resolve contact name: ${e.message}")
        }
        return phoneNumber
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                NOTIFICATION_CHANNEL_ID,
                "C Note Live Call Assistant",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Shows live transcription and AI insights during active phone calls"
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun buildForegroundNotification(contentText: String): Notification {
        return NotificationCompat.Builder(this, NOTIFICATION_CHANNEL_ID)
            .setContentTitle("C Note Active Call Assistant")
            .setContentText(contentText)
            .setSmallIcon(android.R.drawable.ic_btn_speak_now)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()
    }

    private fun updateNotification(text: String) {
        val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.notify(NOTIFICATION_ID, buildForegroundNotification(text))
    }

    override fun onDestroy() {
        stopStreamingAndFinalize()
        super.onDestroy()
    }
}
