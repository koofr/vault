import Foundation

public class SubscriptionCallbackFn: SubscriptionCallback {
    private var fn: () -> Void

    public init(_ fn: @escaping () -> Void) {
        self.fn = fn
    }

    public func onChange() {
        DispatchQueue.main.async {
            self.fn()
        }
    }
}

public class TransfersDownloadOpenFn: TransfersDownloadOpen {
    private var fn: (String) -> Void

    public init(_ fn: @escaping (String) -> Void) {
        self.fn = fn
    }

    public func onOpen(localFilePath: String, contentType: String?) {
        DispatchQueue.main.async {
            self.fn(localFilePath)
        }
    }
}

public class TransfersDownloadDoneFn: TransfersDownloadDone {
    private var fn: (String) -> Void

    public init(_ fn: @escaping (String) -> Void) {
        self.fn = fn
    }

    public func onDone(localFilePath: String, contentType: String?) {
        DispatchQueue.main.async {
            self.fn(localFilePath)
        }
    }
}

public class RepoFilesBrowserDirCreatedFn: RepoFilesBrowserDirCreated {
    private var fn: (String) -> Void

    public init(_ fn: @escaping (String) -> Void) {
        self.fn = fn
    }

    public func onCreated(encryptedPath: String) {
        DispatchQueue.main.async {
            self.fn(encryptedPath)
        }
    }
}

public class RemoteFilesBrowserDirCreatedFn: RemoteFilesBrowserDirCreated {
    private var fn: (String) -> Void

    public init(_ fn: @escaping (String) -> Void) {
        self.fn = fn
    }

    public func onCreated(location: String) {
        DispatchQueue.main.async {
            self.fn(location)
        }
    }
}

public class RepoRemovedFn: RepoRemoved {
    private var fn: () -> Void

    public init(_ fn: @escaping () -> Void) {
        self.fn = fn
    }

    public func onRemoved() {
        DispatchQueue.main.async {
            self.fn()
        }
    }
}

public class RepoUnlockUnlockedFn: RepoUnlockUnlocked {
    private var fn: () -> Void

    public init(_ fn: @escaping () -> Void) {
        self.fn = fn
    }

    public func onUnlocked() {
        DispatchQueue.main.async {
            self.fn()
        }
    }
}

public class OAuth2FinishFlowDoneFn: OAuth2FinishFlowDone {
    private var fn: () -> Void

    public init(_ fn: @escaping () -> Void) {
        self.fn = fn
    }

    public func onDone() {
        DispatchQueue.main.async {
            self.fn()
        }
    }
}
