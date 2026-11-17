// Encode recorded PNG frames into an H.264 MP4 with AVFoundation (macOS only,
// no ffmpeg needed).
//
// swiftc -O demos/encode_mp4.swift -o encode_mp4
// ./encode_mp4 <frames-dir> <out.mp4> [fps=30] [repeat=1] [bitrate-mbps=16]
//
// `repeat` plays the seamless loop several times so the video runs longer.

import AVFoundation
import CoreGraphics
import Foundation
import ImageIO

let args = CommandLine.arguments
guard args.count >= 3 else {
    print("usage: encode_mp4 <frames-dir> <out.mp4> [fps] [repeat] [bitrate-mbps]")
    exit(2)
}
let framesDir = URL(fileURLWithPath: args[1])
let outURL = URL(fileURLWithPath: args[2])
let fps = Int32(args.count > 3 ? Int(args[3]) ?? 30 : 30)
let repeats = args.count > 4 ? Int(args[4]) ?? 1 : 1
let mbps = args.count > 5 ? Double(args[5]) ?? 16 : 16

let frames = try FileManager.default.contentsOfDirectory(at: framesDir, includingPropertiesForKeys: nil)
    .filter { $0.lastPathComponent.hasPrefix("frame-") && $0.pathExtension == "png" }
    .sorted { $0.lastPathComponent < $1.lastPathComponent }
guard let first = frames.first,
      let src = CGImageSourceCreateWithURL(first as CFURL, nil),
      let firstImage = CGImageSourceCreateImageAtIndex(src, 0, nil)
else {
    print("no frame-*.png in \(framesDir.path)")
    exit(1)
}
let width = firstImage.width
let height = firstImage.height

try? FileManager.default.removeItem(at: outURL)
let writer = try AVAssetWriter(outputURL: outURL, fileType: .mp4)
let input = AVAssetWriterInput(mediaType: .video, outputSettings: [
    AVVideoCodecKey: AVVideoCodecType.h264,
    AVVideoWidthKey: width,
    AVVideoHeightKey: height,
    AVVideoCompressionPropertiesKey: [
        AVVideoAverageBitRateKey: Int(mbps * 1_000_000),
        AVVideoProfileLevelKey: AVVideoProfileLevelH264HighAutoLevel,
        AVVideoMaxKeyFrameIntervalKey: Int(fps) * 2,
    ],
])
input.expectsMediaDataInRealTime = false
let adaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: input, sourcePixelBufferAttributes: [
    kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
    kCVPixelBufferWidthKey as String: width,
    kCVPixelBufferHeightKey as String: height,
])
writer.add(input)
writer.startWriting()
writer.startSession(atSourceTime: .zero)

func pixelBuffer(_ image: CGImage) -> CVPixelBuffer? {
    guard let pool = adaptor.pixelBufferPool else { return nil }
    var buffer: CVPixelBuffer?
    CVPixelBufferPoolCreatePixelBuffer(nil, pool, &buffer)
    guard let pb = buffer else { return nil }
    CVPixelBufferLockBaseAddress(pb, [])
    defer { CVPixelBufferUnlockBaseAddress(pb, []) }
    guard let ctx = CGContext(
        data: CVPixelBufferGetBaseAddress(pb), width: width, height: height, bitsPerComponent: 8,
        bytesPerRow: CVPixelBufferGetBytesPerRow(pb), space: CGColorSpace(name: CGColorSpace.sRGB)!,
        bitmapInfo: CGImageAlphaInfo.noneSkipFirst.rawValue | CGBitmapInfo.byteOrder32Little.rawValue
    ) else { return nil }
    ctx.draw(image, in: CGRect(x: 0, y: 0, width: width, height: height))
    return pb
}

var index: Int64 = 0
for _ in 0..<repeats {
    for url in frames {
        autoreleasepool {
            guard let s = CGImageSourceCreateWithURL(url as CFURL, nil),
                  let image = CGImageSourceCreateImageAtIndex(s, 0, nil),
                  let pb = pixelBuffer(image)
            else { print("skipped \(url.lastPathComponent)"); return }
            while !input.isReadyForMoreMediaData { usleep(1000) }
            adaptor.append(pb, withPresentationTime: CMTime(value: index, timescale: fps))
            index += 1
        }
    }
}
input.markAsFinished()
let done = DispatchSemaphore(value: 0)
writer.finishWriting { done.signal() }
done.wait()
if writer.status != .completed {
    print("failed: \(writer.error?.localizedDescription ?? "unknown")")
    exit(1)
}
let size = (try? FileManager.default.attributesOfItem(atPath: outURL.path)[.size] as? Int) ?? 0
print("\(outURL.lastPathComponent): \(index) frames, \(width)x\(height), \(fps) fps, \(size / 1024) KB")
