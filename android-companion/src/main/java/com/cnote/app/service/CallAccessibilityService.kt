package com.cnote.app.service

import android.accessibilityservice.AccessibilityService
import android.content.Intent
import android.os.Build
import android.telephony.TelephonyManager
import android.util.Log
import android.view.accessibility.AccessibilityEvent

/**
 * Accessibility Service companion for C Note.
 *
 * Provides deep in-call event detection and audio pipeline assistance
 * for devices running Android 10+ (API 29+) where standard telephony broadcasts
 * may be delayed or restricted by OEM battery savers.
 */
class CallAccessibilityService : AccessibilityService() {

    companion object {
        private const val TAG = "CNoteAccessibility"
        private const val IN_CALL_UI_PACKAGE = "com.google.android.dialer"
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        val packageName = event?.packageName?.toString() ?: return

        // Detect in-call dialer window active
        if (packageName.contains("dialer") || packageName.contains("incallui")) {
            Log.d(TAG, "In-call screen active: $packageName")
        }
    }

    override fun onInterrupt() {
        Log.w(TAG, "C Note Accessibility Service interrupted")
    }

    override fun onServiceConnected() {
        super.onServiceConnected()
        Log.d(TAG, "C Note Accessibility Service connected and active")
    }
}
