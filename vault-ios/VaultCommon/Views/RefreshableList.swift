import SwiftUI
import VaultMobile

public struct RefreshableList<Empty, Content>: View
where Empty: View, Content: View {
    let status: Status
    let isEmpty: Bool
    let onRefresh: (() -> Void)?
    let empty: () -> Empty
    let content: () -> Content

    public init(
        status: Status, isEmpty: Bool, onRefresh: (() -> Void)?,
        @ViewBuilder empty: @escaping () -> Empty, @ViewBuilder content: @escaping () -> Content
    ) {
        self.status = status
        self.isEmpty = isEmpty
        self.onRefresh = onRefresh
        self.empty = empty
        self.content = content
    }

    public var body: some View {
        VStack {
            switch status {
            case .initial, .loading(loaded: false):
                List {
                    EmptyView()
                }

            case .loading(loaded: true), .loaded, .err(error: _, loaded: true):
                // error will be displayed in notifications. we do not want to
                // hide the current loaded data
                if isEmpty {
                    FullSizeListItem {
                        empty()
                    }
                } else {
                    content()
                }

            case .err(let error, loaded: false):
                FullSizeListItem {
                    ErrorView(errorText: error, onRetry: onRefresh)
                }
            }
        }
        .refreshable {
            if let onRefresh = onRefresh {
                onRefresh()
            }
        }
        .overlay {
            if status == .loading(loaded: false) {
                LoadingView()
            }
        }
    }
}
