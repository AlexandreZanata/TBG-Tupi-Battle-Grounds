package io.github.alexandrezanata.tbg.prototype

import android.text.InputFilter
import android.text.InputType
import android.text.method.PasswordTransformationMethod
import android.view.View
import android.view.WindowManager
import android.view.inputmethod.EditorInfo
import android.widget.CheckBox
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.TextView
import androidx.appcompat.app.AlertDialog
import org.json.JSONObject

/** A system IME modal; Rust supplies localized labels and validates committed text. */
internal class NativeTextInput(private val activity: MainActivity) {
    private var dialog: AlertDialog? = null

    fun show(raw: String): Boolean {
        if (raw.length > 16384) return false
        val request = try { JSONObject(raw) } catch (_: org.json.JSONException) { return false }
        val id = request.optLong("id", -1)
        val limit = request.optInt("limit", 0)
        if (id < 0 || limit !in 1..4096) return false
        for (key in listOf("heading", "title", "done", "cancel", "reveal", "reveal_short")) {
            if (request.optString(key).length !in 1..256) return false
        }
        activity.runOnUiThread { open(request, id, limit) }
        return true
    }

    private fun open(request: JSONObject, id: Long, limit: Int) {
        dialog?.cancel()
        val secret = request.optBoolean("secret", false)
        val field = EditText(activity).apply {
            isSingleLine = true
            textSize = 16f
            isSaveEnabled = false
            importantForAutofill = View.IMPORTANT_FOR_AUTOFILL_NO_EXCLUDE_DESCENDANTS
            inputType = InputType.TYPE_CLASS_TEXT or if (secret) {
                InputType.TYPE_TEXT_VARIATION_PASSWORD
            } else {
                InputType.TYPE_TEXT_FLAG_CAP_SENTENCES
            }
            imeOptions = EditorInfo.IME_ACTION_DONE or EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING or
                EditorInfo.IME_FLAG_NO_EXTRACT_UI or EditorInfo.IME_FLAG_NO_FULLSCREEN
            filters = arrayOf(InputFilter.LengthFilter(limit))
            setText(request.optString("value").take(limit))
            setSelection(text.length)
        }
        val density = activity.resources.displayMetrics.density
        val prompt = TextView(activity).apply {
            text = request.getString("title")
            textSize = 14f
            maxLines = 2
            movementMethod = android.text.method.ScrollingMovementMethod.getInstance()
            isVerticalScrollBarEnabled = true
        }
        val cancel = android.widget.Button(activity).apply {
            text = request.getString("cancel")
            minWidth = (48 * density).toInt()
            textSize = 14f
        }
        val done = android.widget.Button(activity).apply {
            text = request.getString("done")
            minWidth = (48 * density).toInt()
            textSize = 14f
        }
        field.hint = request.getString("heading")
        field.contentDescription = request.getString("title")
        // A short landscape IME can leave only ~110dp. Keep the field and all
        // essential actions on one 48dp row; the full prompt scrolls above it.
        val content = LinearLayout(activity).apply {
            orientation = LinearLayout.VERTICAL
            setPadding((4 * density).toInt(), (4 * density).toInt(),
                (4 * density).toInt(), (4 * density).toInt())
            addView(prompt)
            addView(LinearLayout(activity).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = android.view.Gravity.CENTER_VERTICAL
                val rowHeight = (48 * density).toInt()
                addView(field, LinearLayout.LayoutParams(0, rowHeight, 1f))
                if (secret) addView(CheckBox(activity).apply {
                    text = request.getString("reveal_short")
                    contentDescription = request.getString("reveal")
                    minHeight = rowHeight
                    minWidth = (48 * density).toInt()
                    setOnCheckedChangeListener { _, checked ->
                        field.transformationMethod = if (checked) null else PasswordTransformationMethod.getInstance()
                        field.setSelection(field.text.length)
                    }
                }, LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, rowHeight))
                addView(cancel, LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, rowHeight))
                addView(done, LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, rowHeight))
            })
        }
        var submitted = false
        val modal = AlertDialog.Builder(activity)
            .setView(content)
            .create()
        fun submit() {
            if (submitted) return
            val accepted = activity.events.offer(JSONObject().put("type", "text-commit")
                .put("id", id).put("text", field.text.toString()))
            if (accepted) {
                submitted = true
                modal.dismiss()
            }
        }
        modal.setOnDismissListener {
            if (!submitted) activity.events.offer(JSONObject().put("type", "text-cancel").put("id", id))
            field.text.clear()
            if (dialog === modal) dialog = null
        }
        cancel.setOnClickListener { modal.cancel() }
        done.setOnClickListener { submit() }
        modal.setOnShowListener {
            field.setOnEditorActionListener { _, action, _ ->
                if (action == EditorInfo.IME_ACTION_DONE) { submit(); true } else false
            }
            field.requestFocus()
            field.post {
                activity.getSystemService(android.view.inputmethod.InputMethodManager::class.java)
                    .showSoftInput(field, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT)
            }
            modal.window?.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_STATE_ALWAYS_VISIBLE or
                WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)
        }
        dialog = modal
        modal.show()
        modal.window?.setLayout(WindowManager.LayoutParams.MATCH_PARENT, WindowManager.LayoutParams.WRAP_CONTENT)
    }

    fun dismiss() { activity.runOnUiThread { dialog?.cancel() } }
}
