package ai.vxb.neptune.wallet

import android.graphics.Color
import android.os.Bundle
import android.view.KeyEvent
import android.webkit.WebView
import androidx.core.view.WindowCompat


class MainActivity : TauriActivity() {
  private lateinit var mWebView: RustWebView

  override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
    if (keyCode == KeyEvent.KEYCODE_BACK) {
      if (mWebView.canGoBack()) {
        mWebView.goBack()
      }
      return true
    }
    return super.onKeyDown(keyCode, event)
  }

  override fun onWebViewCreate(webView: WebView) {
    mWebView = webView as RustWebView
    super.onWebViewCreate(webView)
  }

  private fun setFullscreen() {
    WindowCompat.setDecorFitsSystemWindows(window, false)
    window.statusBarColor = Color.TRANSPARENT
    window.navigationBarColor = Color.TRANSPARENT
    WindowCompat.getInsetsController(window, findViewById(android.R.id.content)).let { controller ->
      controller.isAppearanceLightStatusBars = false
      controller.isAppearanceLightNavigationBars = false
    }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    setFullscreen()
  }

}
