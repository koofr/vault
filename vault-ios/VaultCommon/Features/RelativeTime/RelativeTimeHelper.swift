import Foundation
import VaultMobile

public class RelativeTimeHelper: ObservableObject {
    public let mobileVault: MobileVault
    public let value: Int64?
    public let withModifier: Bool

    @Published public var display: String?
    public var nextUpdate: Int64?

    public init(mobileVault: MobileVault, value: Int64?, withModifier: Bool = true) {
        self.mobileVault = mobileVault
        self.value = value
        self.withModifier = withModifier

        if let value = value {
            let relativeTime = mobileVault.relativeTime(
                value: min(value, nowMs()), withModifier: withModifier)
            display = relativeTime.display
            nextUpdate = relativeTime.nextUpdate
        } else {
            display = nil
            nextUpdate = nil
        }
    }

    public func updateLoop() async {
        while value != nil && nextUpdate != nil {
            do {
                try await Task.sleep(for: Duration.milliseconds(max(nextUpdate! - nowMs(), 0)))
            } catch {
                return
            }

            let relativeTime = mobileVault.relativeTime(
                value: min(value!, nowMs()), withModifier: withModifier)

            await MainActor.run {
                display = relativeTime.display
                nextUpdate = relativeTime.nextUpdate
            }
        }
    }
}
