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
            "VAULT_REPOS_SET_DEFAULT_AUTO_LOCK"],
        textEditorAutosaveIntervalMs: parseTextEditorAutosaveIntervalMs(
            ProcessInfo.processInfo.environment[
                "VAULT_TEXT_EDITOR_AUTOSAVE_INTERVAL_MS"])
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

    static func parseTextEditorAutosaveIntervalMs(_ textEditorAutosaveIntervalMsString: String?)
        -> UInt32
    {
        if let valueString = textEditorAutosaveIntervalMsString {
            if let value = UInt32(valueString) {
                return value
            }
        }
        return 20000
    }
}
