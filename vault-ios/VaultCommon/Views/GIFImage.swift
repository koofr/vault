// based on https://github.com/globulus/swiftui-gif/blob/89af6b7e04b0dc0bf97a721d27fc87c4db8e3829/Sources/SwiftUIGIF/SwiftUIGIF.swift

import Foundation

public func gifImage(data: Data) -> UIImage? {
    guard let source = CGImageSourceCreateWithData(data as CFData, nil)
    else {
        return nil
    }
    let count = CGImageSourceGetCount(source)
    let delays = (0..<count).map {
        // store in ms and truncate to compute GCD more easily
        Int(delayForImage(at: $0, source: source) * 1000)
    }
    let duration = delays.reduce(0, +)
    let gcd = delays.reduce(0, gcd)

    var frames = [UIImage]()
    for i in 0..<count {
        if let cgImage = CGImageSourceCreateImageAtIndex(source, i, nil) {
            let frame = UIImage(cgImage: cgImage)
            let frameCount = delays[i] / gcd

            for _ in 0..<frameCount {
                frames.append(frame)
            }
        } else {
            return nil
        }
    }

    return UIImage.animatedImage(
        with: frames,
        duration: Double(duration) / 1000.0)
}

private func gcd(_ a: Int, _ b: Int) -> Int {
    let absB = abs(b)
    let r = abs(a) % absB
    if r != 0 {
        return gcd(absB, r)
    } else {
        return absB
    }
}

private func delayForImage(at index: Int, source: CGImageSource) -> Double {
    let defaultDelay = 1.0

    let cfProperties = CGImageSourceCopyPropertiesAtIndex(source, index, nil)
    let gifPropertiesPointer = UnsafeMutablePointer<UnsafeRawPointer?>.allocate(capacity: 0)
    defer {
        gifPropertiesPointer.deallocate()
    }
    let unsafePointer = Unmanaged.passUnretained(kCGImagePropertyGIFDictionary).toOpaque()
    if CFDictionaryGetValueIfPresent(cfProperties, unsafePointer, gifPropertiesPointer) == false {
        return defaultDelay
    }
    let gifProperties = unsafeBitCast(gifPropertiesPointer.pointee, to: CFDictionary.self)
    var delayWrapper = unsafeBitCast(
        CFDictionaryGetValue(
            gifProperties,
            Unmanaged.passUnretained(kCGImagePropertyGIFUnclampedDelayTime).toOpaque()),
        to: AnyObject.self)
    if delayWrapper.doubleValue == 0 {
        delayWrapper = unsafeBitCast(
            CFDictionaryGetValue(
                gifProperties,
                Unmanaged.passUnretained(kCGImagePropertyGIFDelayTime).toOpaque()),
            to: AnyObject.self)
    }

    if let delay = delayWrapper as? Double,
        delay > 0
    {
        return delay
    } else {
        return defaultDelay
    }
}
