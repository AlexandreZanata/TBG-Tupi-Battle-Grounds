package io.github.alexandrezanata.tbg.prototype

import android.app.Activity
import android.content.Intent
import android.provider.OpenableColumns
import org.json.JSONObject

/** Explicit bounded emoji import through the system picker, retained only in process memory. */
internal class ImagePicker(private val activity: MainActivity) {
    private var pending: Long? = null
    private val generation = java.util.concurrent.atomic.AtomicLong(0)
    @Volatile private var bytes: ByteArray? = null

    fun start(raw: String): Boolean {
        val id = raw.toLongOrNull()?.takeIf { it > 0 } ?: return false
        activity.runOnUiThread {
            if (pending != null) { finish(id, "", false); return@runOnUiThread }
            pending = id
            generation.incrementAndGet()
            bytes = null
            try {
                activity.startActivityForResult(Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                    addCategory(Intent.CATEGORY_OPENABLE)
                    type = "image/*"
                    putExtra(Intent.EXTRA_MIME_TYPES, arrayOf("image/png", "image/gif", "image/webp"))
                }, REQUEST)
            } catch (_: android.content.ActivityNotFoundException) {
                pending = null; finish(id, "", false)
            } catch (_: SecurityException) {
                pending = null; finish(id, "", false)
            }
        }
        return true
    }

    fun result(request: Int, result: Int, data: Intent?): Boolean {
        if (request != REQUEST) return false
        val id = pending ?: return true
        val token = generation.get()
        val uri = data?.data
        if (result != Activity.RESULT_OK || uri == null) {
            pending = null; finish(id, "", false); return true
        }
        Thread {
            var name = "emoji.png"
            val selected = try {
                activity.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use {
                    if (it.moveToFirst()) name = it.getString(0).orEmpty().take(128)
                }
                activity.contentResolver.openInputStream(uri)?.use { input ->
                    val output = java.io.ByteArrayOutputStream()
                    val buffer = ByteArray(8192)
                    var count = input.read(buffer)
                    while (count >= 0) {
                        if (output.size() + count > 1048576) return@use null
                        output.write(buffer, 0, count)
                        count = input.read(buffer)
                    }
                    output.toByteArray().takeIf { it.isNotEmpty() }
                }
            } catch (_: java.io.IOException) { null } catch (_: SecurityException) { null }
            activity.runOnUiThread {
                if (generation.get() == token) {
                    pending = null
                    bytes = selected
                    finish(id, name, selected != null)
                }
            }
        }.start()
        return true
    }

    @Synchronized fun take(): ByteArray? = bytes.also { bytes = null }
    fun clear() {
        generation.incrementAndGet()
        pending?.let { finish(it, "", false) }; pending = null; bytes = null
    }
    private fun finish(id: Long, name: String, success: Boolean) {
        activity.events.offer(JSONObject().put("type", "image-picked").put("id", id)
            .put("name", name).put("success", success))
    }
    companion object { private const val REQUEST = 0x694c }
}
