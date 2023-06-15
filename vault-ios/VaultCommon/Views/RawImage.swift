import SwiftUI

public struct RawImage: UIViewRepresentable {
    private let image: UIImage

    public init(image: UIImage) {
        self.image = image
    }

    public func makeUIView(context: Context) -> RawImageView {
        let imageView = RawImageView()

        imageView.contentMode = .scaleAspectFit
        imageView.image = image

        imageView.translatesAutoresizingMaskIntoConstraints = false
        imageView.setContentHuggingPriority(.defaultHigh, for: .vertical)
        imageView.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        imageView.setContentCompressionResistancePriority(.defaultLow, for: .vertical)
        imageView.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)

        return imageView
    }

    public func updateUIView(_ imageView: RawImageView, context: Context) {
        imageView.image = image
    }

    public class RawImageView: UIImageView {
        public override func layoutSubviews() {
            super.layoutSubviews()

            if let superview = superview {
                if let image = image {
                    frame = calculateFrame(imageSize: image.size, viewSize: superview.bounds.size)
                }
            }
        }

        private func calculateFrame(imageSize: CGSize, viewSize: CGSize) -> CGRect {
            if imageSize.width < viewSize.width && imageSize.height < viewSize.height {
                let x = (viewSize.width - imageSize.width) / 2
                let y = (viewSize.height - imageSize.height) / 2
                return CGRect(x: x, y: y, width: imageSize.width, height: imageSize.height)
            } else {
                let scaleFactor = min(
                    viewSize.width / imageSize.width, viewSize.height / imageSize.height)
                let newWidth = imageSize.width * scaleFactor
                let newHeight = imageSize.height * scaleFactor
                let x = (viewSize.width - newWidth) / 2
                let y = (viewSize.height - newHeight) / 2
                return CGRect(x: x, y: y, width: newWidth, height: newHeight)
            }
        }
    }
}
