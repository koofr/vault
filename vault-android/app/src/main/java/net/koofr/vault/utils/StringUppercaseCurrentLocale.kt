package net.koofr.vault.utils

import java.util.Locale

fun String.uppercaseCurrentLocale(): String = this.uppercase(Locale.getDefault())
