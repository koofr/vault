import Foundation
import XCTest

class BlockOn<T> {
    var result: Result<T, Error>? = nil
}

func blockOn<T>(@_implicitSelfCapture _ operation: @Sendable @escaping () async throws -> T) throws
    -> T
{
    let blockOn = BlockOn<T>()

    Task {
        let task = Task(operation: operation)
        blockOn.result = await task.result
    }

    DispatchQueue.global().sync {
        while blockOn.result == nil {
            RunLoop.current.run(mode: .default, before: .now.addingTimeInterval(0.001))
        }
    }

    switch blockOn.result {
    case let .success(value):
        return value
    case let .failure(error):
        throw error
    case .none:
        fatalError("Run blocking not received value")
    }
}

func blockOnFatal<T>(@_implicitSelfCapture _ operation: @Sendable @escaping () async throws -> T)
    -> T
{
    do {
        return try blockOn(operation)
    } catch {
        fatalError("\(error)")
    }
}
