package com.iptv.helper

import android.os.Bundle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    registerPlugin(IntentPlugin::class.java)
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }
}
