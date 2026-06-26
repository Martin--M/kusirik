package com.martinm.kusirik

import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import app.tauri.TauriActivity

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    registerPlugin(IntentPlugin::class.java)
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }
}
