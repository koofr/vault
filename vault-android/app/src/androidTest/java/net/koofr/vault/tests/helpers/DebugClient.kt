package net.koofr.vault.tests.helpers

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.net.URL
import java.security.cert.X509Certificate
import javax.net.ssl.HostnameVerifier
import javax.net.ssl.HttpsURLConnection
import javax.net.ssl.SSLContext
import javax.net.ssl.TrustManager
import javax.net.ssl.X509TrustManager

@Serializable
data class QueuedRequest(
    val method: String,
    val url: String,
)

@Serializable
data class StateResponse(
    val queueEnabled: Boolean,
    val queueRequests: List<QueuedRequest>,
    val pauseEnabled: Boolean,
    val downloadsPauseEnabled: Boolean,
    val uploadsPauseEnabled: Boolean,
)

class DebugClient constructor(private val baseUrl: String) {
    private val sslContext = getInvalidCertsSSLContext()
    private val json = Json { ignoreUnknownKeys = true }

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

    fun queueEnable() {
        request(getConnection("GET", "/debug/queue/enable"))
    }

    fun queueDisable() {
        request(getConnection("GET", "/debug/queue/disable"))
    }

    fun queueNext(status: Int? = null) {
        var url = "/debug/queue/next"
        if (status != null) {
            url += "?status=$status"
        }
        request(getConnection("GET", url))
    }

    fun state(): StateResponse {
        val connection = getConnection("GET", "/debug/state.json")
        val (_, body) = request(connection, 200)
        return json.decodeFromString<StateResponse>(body)
    }

    fun withQueue(
        callback: (QueuedRequest) -> Boolean,
        before: (() -> Unit)? = null,
    ) {
        queueEnable()
        if (before != null) {
            before()
        }
        while (true) {
            val state = state()
            for (request in state.queueRequests) {
                if (!callback(request)) {
                    queueDisable()
                    return
                }
            }
            Thread.sleep(50)
        }
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
