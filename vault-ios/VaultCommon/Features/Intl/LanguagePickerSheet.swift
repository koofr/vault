import SwiftUI
import VaultMobile

public struct LanguagePickerSheet: View {
    @ObservedObject var vm: LanguagePickerViewModel

    @ObservedObject private var locales: Subscription<[IntlLocale]>
    @ObservedObject private var currentLocale: Subscription<IntlLocale>

    public var onDismiss: () -> Void

    public init(vm: LanguagePickerViewModel, onDismiss: @escaping () -> Void) {
        self.vm = vm
        self.onDismiss = onDismiss
        self.locales = vm.locales
        self.currentLocale = vm.currentLocale
    }

    public var body: some View {
        NavigationView {
            Group {
                if let locales = locales.data, let currentLocale = currentLocale.data {
                    List {
                        ForEach(locales, id: \.locale) { item in
                            Button {
                                vm.changeLocale(item)

                                onDismiss()
                            } label: {
                                HStack {
                                    Text(item.name)
                                        .foregroundColor(Color(.label))
                                    Spacer()
                                    if item.locale == currentLocale.locale {
                                        Image(systemName: "checkmark")
                                            .foregroundColor(Color(.systemBlue))
                                    }
                                }
                            }
                        }
                    }
                }
            }
            .navigationTitle(
                Text(
                    LocalizedStringResource(
                        "ios.language_picker.title",
                        defaultValue: "Change language",
                        bundle: #bundle,
                        comment: "Navigation title of the sheet used to pick the app language."
                    )
                )
            )
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        onDismiss()
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.language_picker.dismiss.button",
                                defaultValue: "Dismiss",
                                bundle: #bundle,
                                comment: "Toolbar button that closes the language picker sheet."
                            )
                        )
                        .bold()
                    }
                }
            }
        }
    }
}
