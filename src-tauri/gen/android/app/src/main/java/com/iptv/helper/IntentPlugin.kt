package com.iptv.helper

import android.app.Activity
import android.content.Intent
import android.net.Uri
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@TauriPlugin
class IntentPlugin(activity: Activity) : Plugin(activity) {
    @Command
    fun launchPlayer(invoke: Invoke) {
        val url = invoke.getString("url") ?: return
        val intent = Intent(Intent.ACTION_VIEW).apply {
            setDataAndType(Uri.parse(url), "video/*")
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        activity.startActivity(Intent.createChooser(intent, "Open with"))
        invoke.resolve()
    }
}
