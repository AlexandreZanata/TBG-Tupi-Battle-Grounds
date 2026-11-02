package io.github.alexandrezanata.tbg.prototype

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri

/** OS services only: the game retains validation and every gameplay action. */
internal class PlatformServices(private val activity: MainActivity) {
    fun copy(text: String) {
        if (text.length > 4096) return
        activity.runOnUiThread {
            val clip = ClipData.newPlainText("TBG", text)
            // Hide credentials from clipboard previews; copying still needs an explicit game action.
            clip.description.extras = android.os.PersistableBundle().apply {
                putBoolean("android.content.extra.IS_SENSITIVE", true)
            }
            activity.getSystemService(ClipboardManager::class.java).setPrimaryClip(clip)
        }
    }

    fun pasteText(): String? {
        val manager = activity.getSystemService(ClipboardManager::class.java)
        return manager.primaryClip?.takeIf { it.itemCount > 0 }?.getItemAt(0)?.text
            ?.toString()?.takeIf { it.length <= 4096 }
    }

    fun paste() {
        activity.runOnUiThread {
            val manager = activity.getSystemService(ClipboardManager::class.java)
            // coerceToText may read a content URI; accept plain text only.
            val value = manager.primaryClip?.takeIf { it.itemCount > 0 }
                ?.getItemAt(0)?.text?.toString()?.takeIf { it.length <= 4096 }
            activity.events.offer(org.json.JSONObject().put("type", "paste").put("text", value))
        }
    }

    fun openPublicPage(url: String): Boolean {
        if (url.length > 2048) return false
        val uri = Uri.parse(url)
        if (uri.scheme != "https" || uri.host.isNullOrBlank() || uri.userInfo != null) return false
        // Public navigation never carries credentials, room tokens or deep-link commands.
        if (uri.query != null || uri.fragment != null) return false
        if (uri.host != "github.com" && uri.path !in setOf("/", "/ranking", "/ranking/", "/download", "/patch-notes") &&
            uri.path?.startsWith("/profile/") != true) return false
        activity.runOnUiThread {
            try {
                activity.startActivity(Intent(Intent.ACTION_VIEW, uri).apply {
                    addCategory(Intent.CATEGORY_BROWSABLE)
                })
            } catch (_: android.content.ActivityNotFoundException) {
                activity.events.offer(org.json.JSONObject().put("type", "link-unavailable"))
            }
        }
        return true
    }

    companion object {
        // Only these nonsecret settings may reach Android preferences. AccountKey is excluded.
        private val preferenceKeys = setOf(
            "tbg.lang", "tbg.arcanas", "tbg.stronghold", "tbg.arena",
            "tbg_muted", "audio.volume", "tbg_reduced_motion", "video.low", "practice.difficulty",
        )
    }

    private val preferences get() = activity.getSharedPreferences("tbg-settings", Context.MODE_PRIVATE)

    fun preference(key: String): String? =
        if (key in preferenceKeys) preferences.getString(key, null) else null

    fun savePreference(key: String, value: String?): Boolean {
        if (key !in preferenceKeys || (value != null && value.length > 8192)) return false
        preferences.edit().apply {
            if (value == null) remove(key) else putString(key, value)
        }.apply()
        return true
    }
}
