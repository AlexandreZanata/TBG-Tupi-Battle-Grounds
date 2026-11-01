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
        super.onDestroy()
    }

    override fun onApplyWindowInsets(view: View, insets: WindowInsetsCompat): WindowInsetsCompat {
        // Preserve GameActivity's own input/content-rect callbacks before taking a snapshot.
        val result = super.onApplyWindowInsets(view, insets)
        val surface = mSurfaceView ?: return result
        val safe = insets.getInsets(WindowInsetsCompat.Type.systemBars() or
            WindowInsetsCompat.Type.displayCutout() or WindowInsetsCompat.Type.systemGestures())
        val ime = insets.getInsets(WindowInsetsCompat.Type.ime())
        val margins = surface.layoutParams as? android.view.ViewGroup.MarginLayoutParams
        if (margins != null) {
            val content = if (battleViewport) androidx.core.graphics.Insets.NONE else safe
            val bottom = maxOf(content.bottom, ime.bottom)
            if (margins.leftMargin != content.left || margins.topMargin != content.top ||
                margins.rightMargin != content.right || margins.bottomMargin != bottom) {
                margins.setMargins(content.left, content.top, content.right, bottom)
                surface.layoutParams = margins
            }
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
        val safe = insets.getInsets(WindowInsetsCompat.Type.systemBars() or
            WindowInsetsCompat.Type.displayCutout() or WindowInsetsCompat.Type.systemGestures())
        val ime = insets.getInsets(WindowInsetsCompat.Type.ime())
        val location = IntArray(2)
        surface.getLocationInWindow(location)
        val bounds = windowManager.currentWindowMetrics.bounds
        viewport = JSONObject()
            .put("width_px", bounds.width()).put("height_px", bounds.height())
            .put("surface_width_px", surface.width).put("surface_height_px", surface.height)
            .put("surface_x_px", location[0]).put("surface_y_px", location[1])
            .put("density", resources.displayMetrics.density).put("battle_viewport", battleViewport)
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
            val controller = androidx.core.view.WindowCompat.getInsetsController(window, window.decorView)
            controller.systemBarsBehavior = androidx.core.view.WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            if (enabled) {
                controller.hide(WindowInsetsCompat.Type.systemBars())
                window.addFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
            } else {
                controller.show(WindowInsetsCompat.Type.systemBars())
                window.clearFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
            }
            androidx.core.view.ViewCompat.requestApplyInsets(window.decorView)
            mSurfaceView?.requestApplyInsets()
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
