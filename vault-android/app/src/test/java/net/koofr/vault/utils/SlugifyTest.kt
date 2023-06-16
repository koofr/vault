package net.koofr.vault.utils

import org.junit.Assert
import org.junit.Test

class SlugifyTest {
    @Test
    fun testSlugifySpecial() {
        Assert.assertEquals("test-name", slugify("test#@!name"))
    }

    @Test
    fun testSlugifyNFCNFD() {
        // input NFC
        Assert.assertEquals("o", slugify("ö"))
        // input NFD
        Assert.assertEquals("o", slugify("ö"))
    }

    @Test
    fun testSlugifyWhitespace() {
        Assert.assertEquals("test-name", slugify("test  name"))
    }

    @Test
    fun testSlugifyHyphens() {
        Assert.assertEquals("test-name", slugify("test--name"))
    }

    @Test
    fun testSlugifyLowercase() {
        Assert.assertEquals("test-name", slugify("TEST NAME"))
    }
}
