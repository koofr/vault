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
            SubscriptionCallbackFn { [weak self] in
                self?.update()
            })

        self.data = getData(mobileVault, self.id!)
    }

    private func update() {
        let data = getData(mobileVault, id!)

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
        var id: UInt32?
        var resumed = false

        let cb = {
            let data = getData(mobileVault, id!)

            if let data = data {
                mobileVault.unsubscribe(id: id!)

                if !resumed {
                    resumed = true

                    continuation.resume(returning: data)
                }
            }
        }

        id = subscribe(mobileVault, SubscriptionCallbackFn(cb))

        cb()
    })
}
