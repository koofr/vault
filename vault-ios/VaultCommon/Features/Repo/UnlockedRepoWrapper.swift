import SwiftUI

struct UnlockedRepoWrapper<Content>: View
where Content: View {
    var container: Container
    var repoId: String
    var content: () -> Content

    var repoGestureHandler: RepoGestureHandler

    public init(container: Container, repoId: String, @ViewBuilder content: @escaping () -> Content)
    {
        self.container = container
        self.repoId = repoId
        self.content = content

        container.mobileVault.reposTouchRepo(repoId: repoId)

        repoGestureHandler = RepoGestureHandler(
            onTouch: {
                container.mobileVault.reposTouchRepo(repoId: repoId)
            },
            onLock: {
                container.mobileVault.reposLockRepo(repoId: repoId)
            }
        )
    }

    var body: some View {
        ZStack {
            WindowReader(handler: repoGestureHandler.handleWindow)

            content()
        }
    }
}

class RepoGestureHandler: NSObject, UIGestureRecognizerDelegate {
    let onTouch: () -> Void
    let onLock: () -> Void

    var window: UIWindow?
    var gesture: UILongPressGestureRecognizer?

    init(onTouch: @escaping () -> Void, onLock: @escaping () -> Void) {
        self.onTouch = onTouch
        self.onLock = onLock
    }

    deinit {
        cleanupGesture()
    }

    func handleWindow(_ window: UIWindow?) {
        cleanupGesture()

        if let window = window {
            self.window = window

            setupGesture(window: window)
        }
    }

    @objc
    func handleGesture(_ gestureReconizer: UIGestureRecognizer) {
        if gestureReconizer.state == .began {
            onLock()
        }
    }

    func setupGesture(window: UIWindow) {
        let gesture = UILongPressGestureRecognizer(
            target: self, action: #selector(handleGesture(_:)))
        gesture.minimumPressDuration = 5
        gesture.requiresExclusiveTouchType = false
        gesture.cancelsTouchesInView = false
        gesture.delegate = self

        self.gesture = gesture

        window.addGestureRecognizer(gesture)
    }

    func cleanupGesture() {
        if let window = window {
            if let gesture = gesture {
                window.removeGestureRecognizer(gesture)
            }

            gesture = nil
        }
    }

    public func gestureRecognizer(
        _ gestureRecognizer: UIGestureRecognizer, shouldReceive touch: UITouch
    ) -> Bool {
        onTouch()

        return true
    }

    public func gestureRecognizer(
        _ gestureRecognizer: UIGestureRecognizer,
        shouldRecognizeSimultaneouslyWith otherGestureRecognizer: UIGestureRecognizer
    ) -> Bool {
        return true
    }
}
