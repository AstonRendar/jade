package io.github.astonrendar.jade

import android.net.wifi.WifiManager
import android.os.Build
import android.os.Bundle
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  private val responsiveWifi by lazy {
    val wifi = applicationContext.getSystemService(WIFI_SERVICE) as WifiManager
    wifi.createWifiLock(responsiveWifiMode(), "jade:sync").apply { setReferenceCounted(false) }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    keepContentClearOfSystemBars()
  }

  override fun onResume() {
    super.onResume()
    responsiveWifi.acquire()
  }

  override fun onPause() {
    responsiveWifi.release()
    super.onPause()
  }

  @Suppress("DEPRECATION")
  private fun responsiveWifiMode(): Int =
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) WifiManager.WIFI_MODE_FULL_LOW_LATENCY
    else WifiManager.WIFI_MODE_FULL_HIGH_PERF

  private fun keepContentClearOfSystemBars() {
    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val reserved = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or
          WindowInsetsCompat.Type.displayCutout() or
          WindowInsetsCompat.Type.ime()
      )
      view.setPadding(reserved.left, reserved.top, reserved.right, reserved.bottom)
      WindowInsetsCompat.CONSUMED
    }
  }
}
