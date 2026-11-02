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
        for (key in listOf("title", "done", "cancel", "reveal")) {
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
        val content = LinearLayout(activity).apply {
            orientation = LinearLayout.VERTICAL
            val padding = (16 * resources.displayMetrics.density).toInt()
            setPadding(padding, padding, padding, padding)
            addView(field)
            if (secret) addView(CheckBox(activity).apply {
                text = request.getString("reveal")
                minHeight = (48 * resources.displayMetrics.density).toInt()
                setOnCheckedChangeListener { _, checked ->
                    field.transformationMethod = if (checked) null else PasswordTransformationMethod.getInstance()
                    field.setSelection(field.text.length)
                }
            })
        }
        var submitted = false
        val modal = AlertDialog.Builder(activity)
            .setTitle(request.getString("title"))
            .setView(content)
            .setPositiveButton(request.getString("done"), null)
            .setNegativeButton(request.getString("cancel")) { _, _ -> }
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
        modal.setOnShowListener {
            modal.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener { submit() }
            field.setOnEditorActionListener { _, action, _ ->
                if (action == EditorInfo.IME_ACTION_DONE) { submit(); true } else false
            }
            field.requestFocus()
            modal.window?.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_STATE_ALWAYS_VISIBLE or
                WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)
        }
        dialog = modal
        modal.show()
    }

    fun dismiss() { activity.runOnUiThread { dialog?.cancel() } }
}
