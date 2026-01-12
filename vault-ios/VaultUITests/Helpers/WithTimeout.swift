import Foundation

enum TimeoutError: Error {
    case timedOut
}

func withTimeout<T>(
    seconds: Double,
    operation: @escaping () async throws -> T
) async throws -> T {
    try await withThrowingTaskGroup(of: T.self) { group in
        // Task 1: the actual operation
        group.addTask {
            try await operation()
        }

        // Task 2: the timeout
        group.addTask {
            try await Task.sleep(nanoseconds: UInt64(seconds * 1_000_000_000))
            throw TimeoutError.timedOut
        }

        // First task to finish wins
        let result = try await group.next()!

        // Cancel the losing task
        group.cancelAll()

        return result
    }
}
