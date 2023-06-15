import Foundation

func nowMs() -> Int64 {
    Int64(Date().timeIntervalSince1970) * 1000
}
