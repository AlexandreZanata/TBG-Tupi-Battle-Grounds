package io.github.alexandrezanata.tbg.prototype

import org.json.JSONArray
import org.json.JSONObject

/** Bounded, process-only messages drained by Rust; never log form contents. */
internal class HostEvents {
    private val pending = ArrayDeque<JSONObject>()

    @Synchronized
    fun offer(event: JSONObject): Boolean {
        if (pending.size >= 64) return false
        pending.addLast(event)
        return true
    }

    @Synchronized
    fun drain(): JSONArray {
        val result = JSONArray()
        while (pending.isNotEmpty()) result.put(pending.removeFirst())
        return result
    }
}
