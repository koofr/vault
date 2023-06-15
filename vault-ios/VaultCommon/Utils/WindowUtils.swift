import UIKit

public func getKeyWindow() -> UIWindow? {
    return UIApplication
        .shared
        .connectedScenes
        .compactMap { ($0 as? UIWindowScene)?.keyWindow }
        .last
}

// WindowSubviewRemoved remembers the number of keyWindow's subviews. When
// start() is called it will poll the number of subviews and wait until it gets
// back to the initial subviews count. There is also a fallback duration after
// which the callback will be called if waiting somehow breaks.
public class WindowSubviewRemoved {
    private let window: UIWindow?
    private let initialSubviewsCount: Int?

    private var callback: (() -> Void)?

    public init() {
        window = getKeyWindow()
        initialSubviewsCount = window?.subviews.count
    }

    public func start(fallbackDuration: DispatchTimeInterval, callback: @escaping () -> Void) {
        self.callback = callback

        if window != nil {
            wait()
        }

        DispatchQueue.main.asyncAfter(deadline: DispatchTime.now() + fallbackDuration) {
            self.handleCallback()
        }
    }

    private func wait() {
        if let window = window {
            if let initialSubviewsCount = initialSubviewsCount {
                if callback != nil {
                    if window.subviews.count == initialSubviewsCount {
                        handleCallback()
                    } else {
                        DispatchQueue.main.asyncAfter(deadline: .now() + 0.01) {
                            self.wait()
                        }
                    }
                }
            }
        }
    }

    private func handleCallback() {
        if let callback = self.callback {
            self.callback = nil

            callback()
        }
    }
}
