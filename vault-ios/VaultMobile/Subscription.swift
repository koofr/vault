import Atomics
import Combine
import Foundation

public class Subscription<T>: ObservableObject {
    private var mobileVault: MobileVault
    private var getData: (MobileVault, UInt32) -> T?
    private var id: UInt32?
    private var onData: ((T?) -> Void)?
    @Published public var data: T?

    public init(
        mobileVault: MobileVault, subscribe: (MobileVault, SubscriptionCallback) -> UInt32,
        getData: @escaping (MobileVault, UInt32) -> T?
    ) {
        self.mobileVault = mobileVault
        self.getData = getData

        self.id = subscribe(
            mobileVault,
            SubscriptionCallbackFn { [weak self] id in
                self?.update(id)
            })

        self.data = getData(mobileVault, self.id!)
    }

    private func update(_ id: UInt32) {
        let data = getData(mobileVault, id)

        self.data = data

        if let onData = onData {
            onData(data)
        }
    }

    public func setOnData(_ onData: @escaping (T?) -> Void) {
        self.onData = onData

        onData(data)
    }

    deinit {
        self.mobileVault.unsubscribe(id: id!)
    }
}

public func subscriptionWait<T>(
    mobileVault: MobileVault, subscribe: (MobileVault, SubscriptionCallback) -> UInt32,
    getData: @escaping (MobileVault, UInt32) -> T?
) async -> T {
    await withCheckedContinuation({ continuation in
        let resumed = ManagedAtomic(false)

        let cb = { @Sendable (id: UInt32) in
            let data = getData(mobileVault, id)

            if let data = data {
                mobileVault.unsubscribe(id: id)

                let didResume = resumed.compareExchange(
                    expected: false,
                    desired: true,
                    ordering: .acquiringAndReleasing
                ).exchanged

                if didResume {
                    continuation.resume(returning: data)
                }
            }
        }

        let id = subscribe(mobileVault, SubscriptionCallbackFn(cb))

        cb(id)
    })
}
