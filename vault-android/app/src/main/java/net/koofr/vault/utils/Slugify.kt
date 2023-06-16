package net.koofr.vault.utils

import java.text.Normalizer
import java.util.Locale
import java.util.regex.Pattern

private val whitespace = Pattern.compile("\\s")
private val special = Pattern.compile("[^\\p{L}-]")
private val nonLatin = Pattern.compile("[^\\w-]")
private val hyphens = Pattern.compile("-+")
private val leadingHyphens = Pattern.compile("^-+")
private val trailingHyphens = Pattern.compile("-+$")

fun slugify(input: String): String {
    var s = special.matcher(input).replaceAll("-")
    s = Normalizer.normalize(s, Normalizer.Form.NFD)
    s = nonLatin.matcher(s).replaceAll("")
    s = whitespace.matcher(s).replaceAll("-")
    s = hyphens.matcher(s).replaceAll("-")
    s = leadingHyphens.matcher(s).replaceAll("")
    s = trailingHyphens.matcher(s).replaceAll("")
    return s.lowercase(Locale.ROOT)
}
