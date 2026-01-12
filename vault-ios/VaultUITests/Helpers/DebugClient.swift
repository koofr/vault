import Foundation

class DebugClient {
    let baseUrl: String

    init(baseUrl: String) {
        self.baseUrl = baseUrl
    }

    func getRequest(method: String, url: String) -> URLRequest {
        var request = URLRequest(url: URL(string: "\(baseUrl)\(url)")!)
        request.httpMethod = method
        return request
    }

    func request(_ request: URLRequest, expectedStatusCode: Int?) async throws -> (
        Data, HTTPURLResponse
    ) {
        let (data, response) = try await URLSession.shared.data(for: request)

        let httpResponse = response as! HTTPURLResponse

        if let expectedStatusCode = expectedStatusCode {
            if httpResponse.statusCode != expectedStatusCode {
                throw DebugClientError.error(
                    "Expected status code \(expectedStatusCode) got \(httpResponse.statusCode): \(String(decoding: data, as: UTF8.self))"
                )
            }
        }

        return (data, httpResponse)
    }

    func reset() async throws {
        let request = getRequest(method: "GET", url: "/debug/reset")
        let _ = try await self.request(request, expectedStatusCode: 200)
    }

    func createTestVaultRepo() async throws {
        let request = getRequest(method: "GET", url: "/debug/vault/repos/create")
        let _ = try await self.request(request, expectedStatusCode: 200)
    }

    func oauth2Revoke() async throws {
        let request = getRequest(method: "GET", url: "/debug/oauth2/revoke")
        let _ = try await self.request(request, expectedStatusCode: 200)
    }

    func downloadsPause() async throws {
        let request = getRequest(method: "GET", url: "/debug/downloads/pause")
        let _ = try await self.request(request, expectedStatusCode: 200)
    }

    func downloadsResume() async throws {
        let request = getRequest(method: "GET", url: "/debug/downloads/resume")
        let _ = try await self.request(request, expectedStatusCode: 200)
    }

    func uploadsPause() async throws {
        let request = getRequest(method: "GET", url: "/debug/uploads/pause")
        let _ = try await self.request(request, expectedStatusCode: 200)
    }

    func uploadsResume() async throws {
        let request = getRequest(method: "GET", url: "/debug/uploads/resume")
        let _ = try await self.request(request, expectedStatusCode: 200)
    }
}

enum DebugClientError: Error {
    case error(String)
}
