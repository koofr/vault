package net.koofr.vault.tests

import java.net.URL
import java.security.cert.X509Certificate
import javax.net.ssl.HostnameVerifier
import javax.net.ssl.HttpsURLConnection
import javax.net.ssl.SSLContext
import javax.net.ssl.TrustManager
import javax.net.ssl.X509TrustManager

class DebugClient constructor(private val baseUrl: String) {
    private val sslContext = getInvalidCertsSSLContext()

    private fun getConnection(method: String, url: String): HttpsURLConnection {
        val connection = URL("${baseUrl}$url").openConnection() as HttpsURLConnection
        connection.requestMethod = method
        connection.sslSocketFactory = sslContext.socketFactory
        connection.hostnameVerifier = HostnameVerifier { _, _ -> true }
        return connection
    }

    private fun request(
        connection: HttpsURLConnection,
        expectedStatusCode: Int = 200,
    ): Pair<Int, String> {
        try {
            val statusCode = connection.responseCode
            val body = connection.inputStream.bufferedReader().use { it.readText() }

            if (statusCode != expectedStatusCode) {
                throw IllegalStateException("Expected status code $expectedStatusCode got $statusCode: $body")
            }

            return Pair(statusCode, body)
        } finally {
            connection.disconnect()
        }
    }

    fun reset() {
        request(getConnection("GET", "/debug/reset"))
    }

    fun createTestVaultRepo() {
        request(getConnection("GET", "/debug/vault/repos/create"))
    }

    fun oauth2Revoke() {
        request(getConnection("GET", "/debug/oauth2/revoke"))
    }

    fun downloadsPause() {
        request(getConnection("GET", "/debug/downloads/pause"))
    }

    fun downloadsResume() {
        request(getConnection("GET", "/debug/downloads/resume"))
    }

    fun uploadsPause() {
        request(getConnection("GET", "/debug/uploads/pause"))
    }

    fun uploadsResume() {
        request(getConnection("GET", "/debug/uploads/resume"))
    }
}

private fun getInvalidCertsSSLContext(): SSLContext {
    val trustAllCerts = arrayOf<TrustManager>(
        object : X509TrustManager {
            override fun getAcceptedIssuers(): Array<X509Certificate> {
                return arrayOf()
            }

            override fun checkClientTrusted(
                certs: Array<X509Certificate>,
                authType: String,
            ) {
            }

            override fun checkServerTrusted(
                certs: Array<X509Certificate>,
                authType: String,
            ) {
            }
        },
    )

    val sslContext = SSLContext.getInstance("TLS")

    sslContext.init(null, trustAllCerts, null)

    return sslContext
}
