package io.github.alexandrezanata.tbg.prototype

import android.app.Activity
import android.content.Intent
import org.json.JSONObject

/** Explicit recovery-key export through Android's picker; secrets stay in process memory. */
internal class RecoveryExport(private val activity: MainActivity) {
    private data class Pending(val epoch: Long, val name: String, val key: String)
    private var pending: Pending? = null

    fun start(raw: String): Boolean {
        if (raw.length > 1024) return false
        val request = try { JSONObject(raw) } catch (_: org.json.JSONException) { return false }
        val name = request.optString("name")
        val key = request.optString("key")
        val epoch = request.optLong("epoch", -1)
        if (!name.matches(Regex("[A-Za-z0-9_]{1,16}")) || key.length !in 16..128 || epoch < 0) return false
        activity.runOnUiThread {
            val next = Pending(epoch, name, key)
            if (pending != null) {
                finish(next, false, false)
                return@runOnUiThread
            }
            pending = next
            try {
                activity.startActivityForResult(Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
                    addCategory(Intent.CATEGORY_OPENABLE)
                    type = "text/plain"
                    putExtra(Intent.EXTRA_TITLE, "$name-tbg-key.txt")
                }, REQUEST)
            } catch (_: android.content.ActivityNotFoundException) {
                pending = null
                finish(next, false, false)
            } catch (_: SecurityException) {
                pending = null
                finish(next, false, false)
            }
        }
        return true
    }

    fun result(requestCode: Int, resultCode: Int, data: Intent?): Boolean {
        if (requestCode != REQUEST) return false
        val current = pending ?: return true
        pending = null
        if (resultCode != Activity.RESULT_OK) {
            finish(current, false, true)
            return true
        }
        val uri = data?.data
        val saved = try {
            uri != null && activity.contentResolver.openOutputStream(uri, "wt")?.use { output ->
                output.write(current.key.toByteArray(Charsets.UTF_8))
                output.flush()
                true
            } == true
        } catch (_: java.io.IOException) {
            false
        } catch (_: SecurityException) {
            false
        }
        finish(current, saved, false)
        return true
    }

    fun clear() { pending = null }

    private fun finish(value: Pending, saved: Boolean, cancelled: Boolean) {
        activity.events.offer(JSONObject().put("type", "backup-export")
            .put("epoch", value.epoch).put("name", value.name)
            .put("saved", saved).put("cancelled", cancelled))
    }

    companion object { private const val REQUEST = 0x694b }
}
