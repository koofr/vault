package net.koofr.vault.utils

import java.net.URLEncoder

fun queryEscape(value: String): String {
    return URLEncoder.encode(value, "UTF-8").replace("+", "%20")
}
