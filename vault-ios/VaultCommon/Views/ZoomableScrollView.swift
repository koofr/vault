// based on https://github.com/jtbandes/SpacePOD/blob/main/SpacePOD/ZoomableScrollView.swift and https://github.com/1and2papa/CTAssetsPickerController/blob/master/CTAssetsPickerController/CTAssetScrollView.m

import Combine
import SwiftUI

struct ZoomableScrollView<Content: View>: View {
    let content: Content

    @State var doubleTap = PassthroughSubject<CGPoint, Never>()

    init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        ZoomableScrollViewImpl(content: content, doubleTap: doubleTap.eraseToAnyPublisher())
            /// The double tap gesture is a modifier on a SwiftUI wrapper view, rather than just putting a UIGestureRecognizer on the wrapped view,
            /// because SwiftUI and UIKit gesture recognizers don't work together correctly correctly for failure and other interactions.
            .onTapGesture(count: 2, coordinateSpace: .local) { location in
                doubleTap.send(location)
            }
    }
}

private struct ZoomableScrollViewImpl<Content: View>: UIViewControllerRepresentable {
    let content: Content
    let doubleTap: AnyPublisher<CGPoint, Never>

    func makeUIViewController(context: Context) -> ViewController {
        return ViewController(coordinator: context.coordinator, doubleTap: doubleTap)
    }

    func updateUIViewController(_ uiViewController: ViewController, context: Context) {
        uiViewController.update(content: self.content, doubleTap: doubleTap)
    }

    func makeCoordinator() -> Coordinator {
        return Coordinator(hostingController: UIHostingController(rootView: content))
    }

    class ViewController: UIViewController, UIScrollViewDelegate {
        let coordinator: Coordinator
        let scrollView = CenteringScrollView()

        var doubleTapCancellable: Cancellable?
        var updateConstraintsCancellable: Cancellable?

        private var hostedView: UIView {
            coordinator.hostingController.view
        }

        private var contentSizeConstraints: [NSLayoutConstraint] = [] {
            willSet { NSLayoutConstraint.deactivate(contentSizeConstraints) }
            didSet { NSLayoutConstraint.activate(contentSizeConstraints) }
        }

        required init?(coder: NSCoder) {
            fatalError()
        }

        init(coordinator: Coordinator, doubleTap: AnyPublisher<CGPoint, Never>) {
            self.coordinator = coordinator

            super.init(nibName: nil, bundle: nil)

            self.view = scrollView

            scrollView.delegate = self
            scrollView.maximumZoomScale = 10
            scrollView.minimumZoomScale = 1
            scrollView.bouncesZoom = true
            scrollView.showsHorizontalScrollIndicator = false
            scrollView.showsVerticalScrollIndicator = false
            scrollView.clipsToBounds = false

            let hostedView = coordinator.hostingController.view!
            hostedView.translatesAutoresizingMaskIntoConstraints = false
            scrollView.addSubview(hostedView)
            NSLayoutConstraint.activate([
                hostedView.leadingAnchor.constraint(
                    equalTo: scrollView.contentLayoutGuide.leadingAnchor),
                hostedView.trailingAnchor.constraint(
                    equalTo: scrollView.contentLayoutGuide.trailingAnchor),
                hostedView.topAnchor.constraint(equalTo: scrollView.contentLayoutGuide.topAnchor),
                hostedView.bottomAnchor.constraint(
                    equalTo: scrollView.contentLayoutGuide.bottomAnchor),
            ])

            hostedView.isHidden = true

            doubleTapCancellable = doubleTap.sink { [unowned self] location in
                handleDoubleTap(location)
            }

            updateConstraintsCancellable = scrollView.publisher(for: \.bounds).map(\.size)
                .removeDuplicates()
                .sink { [unowned self] size in
                    view.setNeedsUpdateConstraints()
                }
        }

        func update(content: Content, doubleTap: AnyPublisher<CGPoint, Never>) {
            coordinator.hostingController.rootView = content

            scrollView.setNeedsUpdateConstraints()

            doubleTapCancellable = doubleTap.sink { [unowned self] location in
                handleDoubleTap(location)
            }
        }

        private func handleDoubleTap(_ location: CGPoint) {
            if scrollView.zoomScale > scrollView.minimumZoomScale {
                scrollView.zoom(to: hostedView.bounds, animated: true)
            } else {
                scrollView.zoom(to: zoomRectForScale(scale: 4, center: location), animated: true)
            }
        }

        private func zoomRectForScale(scale: CGFloat, center: CGPoint) -> CGRect {
            let center = hostedView.convert(center, from: scrollView)

            var zoomRect = CGRect.zero

            zoomRect.size.width = hostedView.bounds.size.width / scale
            zoomRect.size.height = hostedView.bounds.size.height / scale

            zoomRect.origin.x = center.x - (zoomRect.size.width / 2.0)
            zoomRect.origin.y = center.y - (zoomRect.size.height / 2.0)

            return zoomRect
        }

        override func updateViewConstraints() {
            super.updateViewConstraints()

            let hostedContentSize = coordinator.hostingController.sizeThatFits(in: view.bounds.size)

            contentSizeConstraints = [
                hostedView.widthAnchor.constraint(equalToConstant: hostedContentSize.width),
                hostedView.heightAnchor.constraint(equalToConstant: hostedContentSize.height),
            ]
        }

        override func viewDidAppear(_ animated: Bool) {
            super.viewDidAppear(animated)

            scrollView.zoom(to: hostedView.bounds, animated: false)

            hostedView.isHidden = false
        }

        override func viewDidLayoutSubviews() {
            super.viewDidLayoutSubviews()

            let hostedContentSize = coordinator.hostingController.sizeThatFits(in: view.bounds.size)

            scrollView.minimumZoomScale = min(
                scrollView.bounds.width / hostedContentSize.width,
                scrollView.bounds.height / hostedContentSize.height
            )
        }

        func scrollViewDidZoom(_ scrollView: UIScrollView) {
            // For some reason this is needed in both didZoom and layoutSubviews, thanks to https://medium.com/@ssamadgh/designing-apps-with-scroll-views-part-i-8a7a44a5adf7
            // Sometimes this seems to work (view animates size and position simultaneously from current position to center) and sometimes it does not (position snaps to center immediately, size change animates)
            self.scrollView.centerContent()
        }

        override func viewWillTransition(
            to size: CGSize, with coordinator: UIViewControllerTransitionCoordinator
        ) {
            coordinator.animate { [self] context in
                scrollView.zoom(to: hostedView.bounds, animated: false)
            }
        }

        func viewForZooming(in scrollView: UIScrollView) -> UIView? {
            return hostedView
        }
    }

    class Coordinator: NSObject, UIScrollViewDelegate {
        let hostingController: UIHostingController<Content>

        init(hostingController: UIHostingController<Content>) {
            self.hostingController = hostingController
        }
    }
}

private class CenteringScrollView: UIScrollView {
    override func layoutSubviews() {
        super.layoutSubviews()

        centerContent()
    }

    func centerContent() {
        subviews[0].frame.origin.x = max(0, bounds.width - subviews[0].frame.width) / 2
        subviews[0].frame.origin.y = max(0, bounds.height - subviews[0].frame.height) / 2
    }
}

struct ZoomableScrollView_Previews: PreviewProvider {
    static var previews: some View {
        ZoomableScrollView {
            VStack(spacing: 0) {
                HStack(spacing: 0) {
                    Rectangle().frame(width: 200, height: 200).foregroundColor(.red)
                    Rectangle().frame(width: 200, height: 200).foregroundColor(.blue)
                }
                HStack(spacing: 0) {
                    Rectangle().frame(width: 200, height: 200).foregroundColor(.green)
                    Rectangle().frame(width: 200, height: 200).foregroundColor(.yellow)
                }
            }
        }
    }
}
