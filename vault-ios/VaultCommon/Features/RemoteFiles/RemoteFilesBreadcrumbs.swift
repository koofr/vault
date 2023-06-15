import SwiftUI
import VaultMobile

struct RemoteFilesBreadcrumbs: View {
    public let breadcrumbs: [RemoteFilesBreadcrumb]

    init(breadcrumbs: [RemoteFilesBreadcrumb]) {
        self.breadcrumbs = breadcrumbs
    }

    var body: some View {
        ScrollViewReader { proxy in
            let scrollView = ScrollView(.horizontal) {
                HStack(spacing: 0) {
                    ForEach(breadcrumbs, id: \.id) { breadcrumb in
                        HStack(spacing: 0) {
                            Text(breadcrumb.name)

                            if !breadcrumb.last {
                                Image(systemName: "chevron.forward").padding(.horizontal, 10)
                            }
                        }
                        .id(breadcrumb.id)
                    }
                }
            }
            .onAppear {
                if let breadcrumb = breadcrumbs.last {
                    proxy.scrollTo(breadcrumb.id)
                }
            }
            .onChange(
                of: breadcrumbs,
                perform: { breadcrumbs in
                    if let breadcrumb = breadcrumbs.last {
                        proxy.scrollTo(breadcrumb.id)
                    }
                })

            if #available(iOS 16.4, *) {
                scrollView.scrollBounceBehavior(.basedOnSize, axes: [.horizontal])
            } else {
                scrollView
            }
        }
    }
}

struct RemoteFilesBreadcrumbs_Previews: PreviewProvider {
    static var previews: some View {
        Group {
            HStack {
                RemoteFilesBreadcrumbs(breadcrumbs: [])
                    .padding()
            }.background(Color(.systemGray6))
                .previewDisplayName("0")

            HStack {
                RemoteFilesBreadcrumbs(breadcrumbs: [
                    RemoteFilesBreadcrumb(id: "0", mountId: "", path: "", name: "Koofr", last: true)
                ])
                .padding()
            }.background(Color(.systemGray6))
                .previewDisplayName("1")

            HStack {
                RemoteFilesBreadcrumbs(breadcrumbs: [
                    RemoteFilesBreadcrumb(
                        id: "0", mountId: "", path: "", name: "Koofr", last: false),
                    RemoteFilesBreadcrumb(
                        id: "1", mountId: "", path: "", name: "My safe box", last: true),
                ])
                .padding()
            }.background(Color(.systemGray6))
                .previewDisplayName("2")

            HStack {
                RemoteFilesBreadcrumbs(breadcrumbs: [
                    RemoteFilesBreadcrumb(
                        id: "0", mountId: "", path: "", name: "Koofr", last: false),
                    RemoteFilesBreadcrumb(
                        id: "1", mountId: "", path: "", name: "Lorem", last: false),
                    RemoteFilesBreadcrumb(
                        id: "2", mountId: "", path: "", name: "Ipsum", last: false),
                    RemoteFilesBreadcrumb(
                        id: "3", mountId: "", path: "", name: "Dolor", last: false),
                    RemoteFilesBreadcrumb(id: "4", mountId: "", path: "", name: "Sit", last: false),
                    RemoteFilesBreadcrumb(
                        id: "5", mountId: "", path: "", name: "Amet", last: false),
                    RemoteFilesBreadcrumb(
                        id: "6", mountId: "", path: "", name: "Consectetur", last: false),
                    RemoteFilesBreadcrumb(
                        id: "7", mountId: "", path: "", name: "Adipiscing", last: false),
                    RemoteFilesBreadcrumb(
                        id: "8", mountId: "", path: "", name: "My safe box", last: true),
                ])
                .padding()
            }.background(Color(.systemGray6))
                .previewDisplayName("Overflow")
        }
        .previewLayout(.fixed(width: 300, height: 70))
    }
}
