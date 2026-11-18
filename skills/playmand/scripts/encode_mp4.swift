// Encode PNG frames (and optionally a WAV soundtrack) into an H.264 MP4 with
// AVFoundation, on macOS, with no ffmpeg. Also checks a finished file.
//
// swiftc -O scripts/encode_mp4.swift -o encode_mp4
// ./encode_mp4 <frames-dir> <out.mp4> [--fps 30] [--repeat 1] [--mbps 16] [--audio track.wav]
// ./encode_mp4 --verify <file.mp4>
//
// `--repeat` plays a seamless loop several times. With `--audio`, the sound is
// encoded as AAC and the video ends with whichever track is shorter.

import AVFoundation
import CoreGraphics
import Foundation
import ImageIO

func fail(_ message: String) -> Never {
    FileHandle.standardError.write((message + "\n").data(using: .utf8)!)
    exit(1)
}

var args = Array(CommandLine.arguments.dropFirst())

// MARK: verify

if args.first == "--verify" {
    guard args.count == 2 else { fail("usage: encode_mp4 --verify <file.mp4>") }
    let asset = AVURLAsset(url: URL(fileURLWithPath: args[1]))
    let sem = DispatchSemaphore(value: 0)
    Task {
        do {
            let duration = try await asset.load(.duration).seconds
            print(String(format: "duration %.3f s", duration))
            for track in try await asset.loadTracks(withMediaType: .video) {
                let size = try await track.load(.naturalSize)
                let fps = try await track.load(.nominalFrameRate)
                let frames = try await track.load(.timeRange).duration.seconds * Double(fps)
                let codec = try await track.load(.formatDescriptions).first.map {
                    CMFormatDescriptionGetMediaSubType($0)
                }
                let fourcc = codec.map { c in String(bytes: [24, 16, 8, 0].map { UInt8((c >> $0) & 0xff) }, encoding: .ascii) ?? "?" } ?? "?"
                print(String(format: "video %dx%d, %.2f fps, %.0f frames, %@", Int(size.width), Int(size.height), fps, frames, fourcc))
            }
            let audio = try await asset.loadTracks(withMediaType: .audio)
            if audio.isEmpty { print("audio none") }
            for track in audio {
                let desc = try await track.load(.formatDescriptions).first
                let asbd = desc.flatMap { CMAudioFormatDescriptionGetStreamBasicDescription($0)?.pointee }
                let range = try await track.load(.timeRange)
                print(String(format: "audio %.0f Hz, %d ch, %.3f s", asbd?.mSampleRate ?? 0, asbd?.mChannelsPerFrame ?? 0, range.duration.seconds))
            }
        } catch {
            fail("cannot read: \(error.localizedDescription)")
        }
        sem.signal()
    }
    sem.wait()
    exit(0)
}

// MARK: encode

func option(_ name: String) -> String? {
    guard let i = args.firstIndex(of: name), i + 1 < args.count else { return nil }
    let value = args[i + 1]
    args.removeSubrange(i...(i + 1))
    return value
}
let fps = Int32(option("--fps") ?? "30") ?? 30
let repeats = Int(option("--repeat") ?? "1") ?? 1
let mbps = Double(option("--mbps") ?? "16") ?? 16
let audioPath = option("--audio")
guard args.count == 2 else {
    fail("usage: encode_mp4 <frames-dir> <out.mp4> [--fps 30] [--repeat 1] [--mbps 16] [--audio track.wav]")
}
let framesDir = URL(fileURLWithPath: args[0])
let outURL = URL(fileURLWithPath: args[1])

let frames = try FileManager.default.contentsOfDirectory(at: framesDir, includingPropertiesForKeys: nil)
    .filter { $0.lastPathComponent.hasPrefix("frame-") && $0.pathExtension == "png" }
    .sorted { $0.lastPathComponent < $1.lastPathComponent }
guard let first = frames.first,
      let src = CGImageSourceCreateWithURL(first as CFURL, nil),
      let firstImage = CGImageSourceCreateImageAtIndex(src, 0, nil)
else { fail("no frame-*.png in \(framesDir.path)") }
let width = firstImage.width
let height = firstImage.height
if width % 2 != 0 || height % 2 != 0 { fail("H.264 needs even width and height, got \(width)x\(height)") }

try? FileManager.default.removeItem(at: outURL)
let writer = try AVAssetWriter(outputURL: outURL, fileType: .mp4)
let videoInput = AVAssetWriterInput(mediaType: .video, outputSettings: [
    AVVideoCodecKey: AVVideoCodecType.h264,
    AVVideoWidthKey: width,
    AVVideoHeightKey: height,
    AVVideoCompressionPropertiesKey: [
        AVVideoAverageBitRateKey: Int(mbps * 1_000_000),
        AVVideoProfileLevelKey: AVVideoProfileLevelH264HighAutoLevel,
        AVVideoMaxKeyFrameIntervalKey: Int(fps) * 2,
    ],
])
videoInput.expectsMediaDataInRealTime = false
let adaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: videoInput, sourcePixelBufferAttributes: [
    kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
    kCVPixelBufferWidthKey as String: width,
    kCVPixelBufferHeightKey as String: height,
])
writer.add(videoInput)

// The soundtrack: read the WAV as PCM and let the writer encode AAC.
var audioReader: AVAssetReader?
var audioOutput: AVAssetReaderTrackOutput?
var audioInput: AVAssetWriterInput?
if let audioPath {
    let asset = AVURLAsset(url: URL(fileURLWithPath: audioPath))
    let sem = DispatchSemaphore(value: 0)
    var track: AVAssetTrack?
    Task {
        track = try? await asset.loadTracks(withMediaType: .audio).first
        sem.signal()
    }
    sem.wait()
    guard let track else { fail("no audio track in \(audioPath)") }
    let reader = try AVAssetReader(asset: asset)
    let output = AVAssetReaderTrackOutput(track: track, outputSettings: [AVFormatIDKey: kAudioFormatLinearPCM])
    reader.add(output)
    let input = AVAssetWriterInput(mediaType: .audio, outputSettings: [
        AVFormatIDKey: kAudioFormatMPEG4AAC,
        AVSampleRateKey: 48_000,
        AVNumberOfChannelsKey: 2,
        AVEncoderBitRateKey: 192_000,
    ])
    input.expectsMediaDataInRealTime = false
    writer.add(input)
    (audioReader, audioOutput, audioInput) = (reader, output, input)
}

writer.startWriting()
writer.startSession(atSourceTime: .zero)
audioReader?.startReading()

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

// Both tracks are fed on their own queues as the writer asks for data, so it
// can interleave them.
let group = DispatchGroup()
let total = frames.count * repeats
var index = 0
group.enter()
videoInput.requestMediaDataWhenReady(on: DispatchQueue(label: "video")) {
    while videoInput.isReadyForMoreMediaData {
        if index >= total {
            videoInput.markAsFinished()
            group.leave()
            return
        }
        autoreleasepool {
            let url = frames[index % frames.count]
            if let s = CGImageSourceCreateWithURL(url as CFURL, nil),
               let image = CGImageSourceCreateImageAtIndex(s, 0, nil),
               let pb = pixelBuffer(image)
            {
                adaptor.append(pb, withPresentationTime: CMTime(value: Int64(index), timescale: fps))
            } else {
                print("skipped \(url.lastPathComponent)")
            }
            index += 1
        }
    }
}
let videoEnd = CMTime(value: Int64(total), timescale: fps)
if let audioInput, let audioOutput {
    group.enter()
    audioInput.requestMediaDataWhenReady(on: DispatchQueue(label: "audio")) {
        while audioInput.isReadyForMoreMediaData {
            guard let sample = audioOutput.copyNextSampleBuffer(),
                  CMSampleBufferGetPresentationTimeStamp(sample) < videoEnd
            else {
                audioInput.markAsFinished()
                group.leave()
                return
            }
            audioInput.append(sample)
        }
    }
}
group.wait()
writer.endSession(atSourceTime: videoEnd)
let done = DispatchSemaphore(value: 0)
writer.finishWriting { done.signal() }
done.wait()
if writer.status != .completed { fail("failed: \(writer.error?.localizedDescription ?? "unknown")") }
let size = (try? FileManager.default.attributesOfItem(atPath: outURL.path)[.size] as? Int) ?? 0
print("\(outURL.lastPathComponent): \(total) frames, \(width)x\(height), \(fps) fps, audio \(audioInput == nil ? "none" : "AAC"), \(size / 1024) KB")
