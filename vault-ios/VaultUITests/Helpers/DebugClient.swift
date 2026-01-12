import Foundation

class DebugClient {
    enum DebugClientError: Error {
        case error(String)
    }

    struct QueuedRequest: Codable {
        let method: String
        let url: String
    }

    struct StateResponse: Codable {
        let queueEnabled: Bool
        let queueRequests: [QueuedRequest]
        let pauseEnabled: Bool
        let downloadsPauseEnabled: Bool
        let uploadsPauseEnabled: Bool
    }

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

    func queueEnable() async throws {
        let request = getRequest(method: "GET", url: "/debug/queue/enable")
        let _ = try await self.request(request, expectedStatusCode: 200)
    }

    func queueDisable() async throws {
        let request = getRequest(method: "GET", url: "/debug/queue/disable")
        let _ = try await self.request(request, expectedStatusCode: 200)
    }

    func queueNext(status: Int? = nil) async throws {
        var url = "/debug/queue/next"
        if let status = status {
            url += "?status=\(status)"
        }
        let request = getRequest(method: "GET", url: url)
        let _ = try await self.request(request, expectedStatusCode: 200)
    }

    func state() async throws -> StateResponse {
        let request = getRequest(method: "GET", url: "/debug/state.json")
        let (data, _) = try await self.request(request, expectedStatusCode: 200)
        return try JSONDecoder().decode(StateResponse.self, from: data)
    }

    func withQueue(
        callback: @escaping (QueuedRequest) async throws -> Bool,
        before: (() async throws -> Void)? = nil,
    ) async throws {
        try await queueEnable()

        if let before = before {
            try await before()
        }

        while true {
            let state = try await self.state()

            for request in state.queueRequests {
                if try await !callback(request) {
                    try await self.queueDisable()
                    return
                }
            }

            try await Task.sleep(nanoseconds: 50 * 1_000_000)
        }
    }
}
