package io.github.alexandrezanata.tbg.prototype

import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
import android.os.Bundle
import android.view.View
import androidx.activity.OnBackPressedCallback
import androidx.annotation.Keep
import androidx.core.view.WindowInsetsCompat
import com.google.androidgamesdk.GameActivity
import org.json.JSONObject

/** Thin GameActivity host. Bevy/winit owns the single native event loop. */
@Keep
class MainActivity : GameActivity() {
    internal val events = HostEvents()
    private val services by lazy { PlatformServices(this) }
    private val textInput by lazy { NativeTextInput(this) }
    private val recoveryExport by lazy { RecoveryExport(this) }
    private val imagePicker by lazy { ImagePicker(this) }
    @Volatile private var viewport = "{}"
    @Volatile private var battleViewport = false
    @Volatile private var resumed = false
    @Volatile private var audioFocused = false
    private val audio by lazy { getSystemService(AudioManager::class.java) }
    private val focus by lazy {
        AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
            .setAudioAttributes(AudioAttributes.Builder()
                .setUsage(AudioAttributes.USAGE_GAME)
                .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build())
            .setOnAudioFocusChangeListener { audioFocused = it == AudioManager.AUDIOFOCUS_GAIN }
            .build()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        // Do not restore secret text fields or serialize a match/account into Android state.
        super.onCreate(null)
        volumeControlStream = AudioManager.STREAM_MUSIC
        androidx.core.view.WindowCompat.setDecorFitsSystemWindows(window, false)
        window.attributes = window.attributes.apply {
            layoutInDisplayCutoutMode = android.view.WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_ALWAYS
        }
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() { events.offer(JSONObject().put("type", "back")) }
        })
        immersive()
    }

    override fun onConfigurationChanged(newConfig: android.content.res.Configuration) {
        // Keep the single GameActivity/native loop alive for supported configuration changes.
        super.onConfigurationChanged(newConfig)
        window.decorView.post {
            immersive()
            androidx.core.view.ViewCompat.requestApplyInsets(window.decorView)
            androidx.core.view.ViewCompat.getRootWindowInsets(window.decorView)?.let { captureViewport(it) }
        }
        if (isInternalBuild()) android.util.Log.i("TBGHost", "configuration fontScale=${newConfig.fontScale} densityDpi=${newConfig.densityDpi} orientation=${newConfig.orientation}")
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        // Dialogs and the IME can restore the bars; hide them again when the game regains focus.
        if (hasFocus) immersive()
    }

    /** Landscape play is edge to edge everywhere; a swipe shows transient bars over the game. */
    private fun immersive() {
        val controller = androidx.core.view.WindowCompat.getInsetsController(window, window.decorView)
        controller.systemBarsBehavior = androidx.core.view.WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        controller.hide(WindowInsetsCompat.Type.systemBars())
    }

    override fun onResume() {
        super.onResume()
        resumed = true
        audioFocused = audio.requestAudioFocus(focus) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED
    }

    override fun onPause() {
        resumed = false
        audioFocused = false
        audio.abandonAudioFocusRequest(focus)
        super.onPause()
    }

    override fun onDestroy() {
        textInput.dismiss()
        recoveryExport.clear()
        imagePicker.clear()
        super.onDestroy()
    }

    @Deprecated("Android result callback retained for the explicit document picker")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: android.content.Intent?) {
        if (!recoveryExport.result(requestCode, resultCode, data) &&
            !imagePicker.result(requestCode, resultCode, data)) {
            super.onActivityResult(requestCode, resultCode, data)
        }
    }

    override fun onApplyWindowInsets(view: View, insets: WindowInsetsCompat): WindowInsetsCompat {
        // Preserve GameActivity's own input/content-rect callbacks before taking a snapshot.
        val result = super.onApplyWindowInsets(view, insets)
        // The surface always fills the window: scenery is drawn to every edge and Rust
        // insets only the controls, once. Clear margins left by an older layout.
        val surface = mSurfaceView ?: return result
        val margins = surface.layoutParams as? android.view.ViewGroup.MarginLayoutParams
        if (margins != null && (margins.leftMargin != 0 || margins.topMargin != 0 ||
                margins.rightMargin != 0 || margins.bottomMargin != 0)) {
            margins.setMargins(0, 0, 0, 0)
            surface.layoutParams = margins
        }
        captureViewport(insets)
        return result
    }

    override fun onGlobalLayout() {
        super.onGlobalLayout()
        val insets = androidx.core.view.ViewCompat.getRootWindowInsets(window.decorView) ?: return
        captureViewport(insets)
    }

    private fun captureViewport(insets: WindowInsetsCompat) {
        val surface = mSurfaceView ?: return
        // Controls avoid visible bars, the cutout and the mandatory home/edge gesture
        // strips. Back-gesture zones still accept taps, and text entry is a separate dialog.
        val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
        val cutout = insets.getInsets(WindowInsetsCompat.Type.displayCutout())
        val gestures = insets.getInsets(WindowInsetsCompat.Type.mandatorySystemGestures())
        val visibleSafe = androidx.core.graphics.Insets.max(bars, cutout)
        // Lobby taps remain usable at hidden bar edges; only actual cutouts/visible
        // bars reserve space. Battle controls additionally avoid mandatory drags.
        val safe = if (battleViewport) androidx.core.graphics.Insets.max(visibleSafe, gestures) else visibleSafe
        val ime = insets.getInsets(WindowInsetsCompat.Type.ime())
        val location = IntArray(2)
        surface.getLocationInWindow(location)
        val bounds = windowManager.currentWindowMetrics.bounds
        viewport = JSONObject()
            .put("width_px", bounds.width()).put("height_px", bounds.height())
            .put("surface_width_px", surface.width).put("surface_height_px", surface.height)
            .put("surface_x_px", location[0]).put("surface_y_px", location[1])
            .put("surface_buffer_width_px", surface.holder.surfaceFrame.width())
            .put("surface_buffer_height_px", surface.holder.surfaceFrame.height())
            .put("density", resources.displayMetrics.density)
            .put("font_scale", resources.configuration.fontScale)
            .put("rotation", surface.display?.rotation ?: 0).put("battle_viewport", battleViewport)
            .put("bars_top_px", bars.top).put("bars_bottom_px", bars.bottom)
            .put("cutout_left_px", cutout.left).put("cutout_top_px", cutout.top)
            .put("cutout_right_px", cutout.right).put("cutout_bottom_px", cutout.bottom)
            .put("mandatory_top_px", gestures.top).put("mandatory_bottom_px", gestures.bottom)
            .put("left_px", safe.left).put("top_px", safe.top)
            .put("right_px", safe.right).put("bottom_px", safe.bottom)
            .put("ime_bottom_px", ime.bottom).toString()
    }

    fun setBattleViewport(raw: String): Boolean {
        if (raw != "true" && raw != "false") return false
        val enabled = raw == "true"
        runOnUiThread {
            if (battleViewport == enabled) return@runOnUiThread
            battleViewport = enabled
            // Only the screen-on policy differs between the lobby and a running battle.
            if (enabled) {
                window.addFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
            } else {
                window.clearFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
            }
            immersive()
        }
        return true
    }

    // Rust calls these methods through the AndroidApp Activity reference. Nothing logs payloads.
    fun drainHostEvents(): String = events.drain().toString()
    fun viewportSnapshot(): String = viewport
    fun isHostResumed(): Boolean = resumed
    fun isInternalBuild(): Boolean = (applicationInfo.flags and android.content.pm.ApplicationInfo.FLAG_DEBUGGABLE) != 0
    fun hasAudioFocus(): Boolean = resumed && audioFocused
    fun showTextForm(request: String): Boolean = textInput.show(request)
    fun dismissTextForm() = textInput.dismiss()
    fun exportRecoveryKey(request: String): Boolean = recoveryExport.start(request)
    fun chooseImage(request: String): Boolean = imagePicker.start(request)
    fun takePickedImage(): ByteArray? = imagePicker.take()
    fun copyText(text: String) = services.copy(text)
    fun requestPaste() = services.paste()
    fun pasteText(): String? = services.pasteText()
    fun openPublicPage(url: String): Boolean = services.openPublicPage(url)
    fun readPreference(key: String): String? = services.preference(key)
    fun writePreference(key: String, value: String?): Boolean = services.savePreference(key, value)

    companion object {
        init { System.loadLibrary("tbg_client") }
    }
}
