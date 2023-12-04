import AVFAudio
import SwiftUI
import VaultCommon

@main
struct VaultApp: App {
    let container = Container(
        baseURL: ProcessInfo.processInfo.environment["VAULT_BASE_URL"],
        oauth2AuthBaseURL: ProcessInfo.processInfo.environment["VAULT_OAUTH2_AUTH_BASE_URL"],
        secureStorageJson: ProcessInfo.processInfo.environment["VAULT_SECURE_STORAGE"],
        reposSetDefaultAutoLock: ProcessInfo.processInfo.environment[
            "VAULT_REPOS_SET_DEFAULT_AUTO_LOCK"]
    )
    let lifecycleHandler: LifecycleHandler

    init() {
        lifecycleHandler = LifecycleHandler(container: container)

        do {
            try AVAudioSession.sharedInstance().setCategory(.playback)
        } catch {
            print("VaultApp failed to set AVAudioSession category: \(error)")
        }
    }

    var body: some Scene {
        WindowGroup {
            ContentView(container: container)
        }
    }
}
