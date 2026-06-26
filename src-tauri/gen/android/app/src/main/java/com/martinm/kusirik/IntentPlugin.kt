package com.martinm.kusirik

import android.app.Activity
import android.content.Intent
import android.net.Uri
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@TauriPlugin
class IntentPlugin(private val mContext: Activity) : Plugin(mContext) {
    @Command
    fun launchPlayer(invoke: Invoke) {
        val url = invoke.getArgs().getString("url", null) ?: return
        val intent = Intent(Intent.ACTION_VIEW).apply {
            setDataAndType(Uri.parse(url), "video/*")
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        mContext.startActivity(Intent.createChooser(intent, "Open with"))
        invoke.resolve()
    }
}
