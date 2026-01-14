import Foundation

public final class SubscriptionCallbackFn: SubscriptionCallback {
    private let fn: @Sendable () -> Void

    public init(_ fn: @Sendable @escaping () -> Void) {
        self.fn = fn
    }

    public func onChange() {
        DispatchQueue.main.async {
            self.fn()
        }
    }
}

public final class TransfersDownloadOpenFn: TransfersDownloadOpen {
    private let fn: @Sendable (String) -> Void

    public init(_ fn: @Sendable @escaping (String) -> Void) {
        self.fn = fn
    }

    public func onOpen(localFilePath: String, contentType: String?) {
        DispatchQueue.main.async {
            self.fn(localFilePath)
        }
    }
}

public final class TransfersDownloadDoneFn: TransfersDownloadDone {
    private let fn: @Sendable (String) -> Void

    public init(_ fn: @Sendable @escaping (String) -> Void) {
        self.fn = fn
    }

    public func onDone(localFilePath: String, contentType: String?) {
        DispatchQueue.main.async {
            self.fn(localFilePath)
        }
    }
}

public final class RepoFilesBrowserDirCreatedFn: RepoFilesBrowserDirCreated {
    private let fn: @Sendable (String) -> Void

    public init(_ fn: @Sendable @escaping (String) -> Void) {
        self.fn = fn
    }

    public func onCreated(encryptedPath: String) {
        DispatchQueue.main.async {
            self.fn(encryptedPath)
        }
    }
}

public final class RepoFilesBrowserFileCreatedFn: RepoFilesBrowserFileCreated {
    private let fn: @Sendable (String) -> Void

    public init(_ fn: @Sendable @escaping (String) -> Void) {
        self.fn = fn
    }

    public func onCreated(encryptedPath: String) {
        DispatchQueue.main.async {
            self.fn(encryptedPath)
        }
    }
}

public final class RemoteFilesBrowserDirCreatedFn: RemoteFilesBrowserDirCreated {
    private let fn: @Sendable (String) -> Void

    public init(_ fn: @Sendable @escaping (String) -> Void) {
        self.fn = fn
    }

    public func onCreated(location: String) {
        DispatchQueue.main.async {
            self.fn(location)
        }
    }
}

public final class RepoRemovedFn: RepoRemoved {
    private let fn: @Sendable () -> Void

    public init(_ fn: @Sendable @escaping () -> Void) {
        self.fn = fn
    }

    public func onRemoved() {
        DispatchQueue.main.async {
            self.fn()
        }
    }
}

public final class RepoUnlockUnlockedFn: RepoUnlockUnlocked {
    private let fn: @Sendable () -> Void

    public init(_ fn: @Sendable @escaping () -> Void) {
        self.fn = fn
    }

    public func onUnlocked() {
        DispatchQueue.main.async {
            self.fn()
        }
    }
}

public final class OAuth2FinishFlowDoneFn: OAuth2FinishFlowDone {
    private let fn: @Sendable () -> Void

    public init(_ fn: @Sendable @escaping () -> Void) {
        self.fn = fn
    }

    public func onDone() {
        DispatchQueue.main.async {
            self.fn()
        }
    }
}
