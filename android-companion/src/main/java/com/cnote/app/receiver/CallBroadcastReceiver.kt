package com.cnote.app.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import android.telephony.TelephonyManager
import android.util.Log
import com.cnote.app.service.CallStreamService

class CallBroadcastReceiver : BroadcastReceiver() {

    companion object {
        private const val TAG = "CNoteCallReceiver"
        private var lastState = TelephonyManager.CALL_STATE_IDLE
        private var incomingNumber: String? = null
    }

    override fun onReceive(context: Context, intent: Intent) {
        val action = intent.action ?: return

        if (action == Intent.ACTION_NEW_OUTGOING_CALL) {
            incomingNumber = intent.getStringExtra(Intent.EXTRA_PHONE_NUMBER)
            Log.d(TAG, "Outgoing call to: $incomingNumber")
            return
        }

        if (action == TelephonyManager.ACTION_PHONE_STATE_CHANGED) {
            val stateStr = intent.getStringExtra(TelephonyManager.EXTRA_STATE)
            val number = intent.getStringExtra(TelephonyManager.EXTRA_INCOMING_NUMBER)

            if (!number.isNullOrEmpty()) {
                incomingNumber = number
            }

            val state = when (stateStr) {
                TelephonyManager.EXTRA_STATE_RINGING -> TelephonyManager.CALL_STATE_RINGING
                TelephonyManager.EXTRA_STATE_OFFHOOK -> TelephonyManager.CALL_STATE_OFFHOOK
                TelephonyManager.EXTRA_STATE_IDLE -> TelephonyManager.CALL_STATE_IDLE
                else -> TelephonyManager.CALL_STATE_IDLE
            }

            handleCallStateTransition(context, state, incomingNumber)
            lastState = state
        }
    }

    private fun handleCallStateTransition(context: Context, state: Int, phoneNumber: String?) {
        if (state == lastState) return

        when (state) {
            TelephonyManager.CALL_STATE_RINGING -> {
                Log.d(TAG, "Incoming call ringing from $phoneNumber")
            }

            TelephonyManager.CALL_STATE_OFFHOOK -> {
                // Call started (either incoming answered or outgoing connected)
                Log.d(TAG, "Call connected with $phoneNumber. Starting real-time audio capture service.")
                val serviceIntent = Intent(context, CallStreamService::class.java).apply {
                    action = CallStreamService.ACTION_START_STREAMING
                    putExtra(CallStreamService.EXTRA_PHONE_NUMBER, phoneNumber ?: "Unknown")
                    putExtra(CallStreamService.EXTRA_TIMESTAMP, System.currentTimeMillis())
                }

                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    context.startForegroundService(serviceIntent)
                } else {
                    context.startService(serviceIntent)
                }
            }

            TelephonyManager.CALL_STATE_IDLE -> {
                // Call terminated
                if (lastState == TelephonyManager.CALL_STATE_OFFHOOK) {
                    Log.d(TAG, "Call concluded with $phoneNumber. Stopping audio stream and finalizing notes.")
                    val serviceIntent = Intent(context, CallStreamService::class.java).apply {
                        action = CallStreamService.ACTION_STOP_STREAMING
                    }
                    context.startService(serviceIntent)
                }
            }
        }
    }
}
