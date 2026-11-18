// Cut the subject out of generated images: transparent PNGs ready for a game,
// a video or a page. macOS 14+, Apple's Vision framework, offline and free.
//
// swiftc -O scripts/cutout.swift -o cutout
// ./cutout <image-or-folder>... -o <out-folder> [--trim] [--pad 16] [--max 1024]
//
// --trim crops to the subject (plus --pad pixels); --max scales the longest
// side down. Every input becomes <out-folder>/<name>.png. Prints, per image,
// the subject's share of the frame, so empty or failed cut-outs stand out.

import AppKit
import CoreImage
import Foundation
import Vision

func fail(_ message: String) -> Never {
    FileHandle.standardError.write((message + "\n").data(using: .utf8)!)
    exit(1)
}

var args = Array(CommandLine.arguments.dropFirst())
func option(_ name: String) -> String? {
    guard let i = args.firstIndex(of: name), i + 1 < args.count else { return nil }
    let v = args[i + 1]
    args.removeSubrange(i...(i + 1))
    return v
}
func flag(_ name: String) -> Bool {
    guard let i = args.firstIndex(of: name) else { return false }
    args.remove(at: i)
    return true
}
guard let outDir = option("-o") else {
    fail("usage: cutout <image-or-folder>... -o <out-folder> [--trim] [--pad 16] [--max 1024]")
}
let trim = flag("--trim")
let pad = Int(option("--pad") ?? "16") ?? 16
let maxSide = option("--max").flatMap { Int($0) }
if let maxSide, maxSide <= 0 { fail("--max must be a positive number of pixels") }
guard !args.isEmpty else { fail("no input images") }

let fm = FileManager.default
try? fm.createDirectory(atPath: outDir, withIntermediateDirectories: true)
let exts: Set<String> = ["png", "jpg", "jpeg", "webp", "heic"]
var inputs: [URL] = []
for a in args {
    var isDir: ObjCBool = false
    guard fm.fileExists(atPath: a, isDirectory: &isDir) else { fail("not found: \(a)") }
    if isDir.boolValue {
        let items = (try? fm.contentsOfDirectory(atPath: a)) ?? []
        inputs += items.sorted().map { URL(fileURLWithPath: a).appendingPathComponent($0) }
            .filter { exts.contains($0.pathExtension.lowercased()) }
    } else {
        inputs.append(URL(fileURLWithPath: a))
    }
}

let context = CIContext()
var failures = 0
for url in inputs {
    guard let image = CIImage(contentsOf: url, options: [.applyOrientationProperty: true]) else {
        print("\(url.lastPathComponent): cannot read"); failures += 1; continue
    }
    let request = VNGenerateForegroundInstanceMaskRequest()
    let handler = VNImageRequestHandler(ciImage: image)
    do {
        try handler.perform([request])
    } catch {
        print("\(url.lastPathComponent): vision failed: \(error.localizedDescription)"); failures += 1; continue
    }
    guard let observation = request.results?.first, !observation.allInstances.isEmpty else {
        print("\(url.lastPathComponent): no subject found"); failures += 1; continue
    }
    // All foreground instances together: a character and the sword it holds.
    let maskBuffer: CVPixelBuffer
    do {
        maskBuffer = try observation.generateScaledMaskForImage(forInstances: observation.allInstances, from: handler)
    } catch {
        print("\(url.lastPathComponent): mask failed: \(error.localizedDescription)"); failures += 1; continue
    }
    let mask = CIImage(cvPixelBuffer: maskBuffer)
    let clear = CIImage(color: .clear).cropped(to: image.extent)
    var cut = image.applyingFilter("CIBlendWithMask", parameters: [
        kCIInputBackgroundImageKey: clear,
        kCIInputMaskImageKey: mask,
    ])

    // The subject's bounds, from the mask (rendered to 8-bit grey; the mask
    // itself is a float buffer).
    let w = Int(mask.extent.width), h = Int(mask.extent.height)
    var gray = [UInt8](repeating: 0, count: w * h)
    context.render(mask, toBitmap: &gray, rowBytes: w, bounds: mask.extent, format: .L8, colorSpace: nil)
    var minX = w, minY = h, maxX = -1, maxY = -1, covered = 0
    for y in 0..<h {
        for x in 0..<w where gray[y * w + x] > 127 {
            covered += 1
            minX = min(minX, x); maxX = max(maxX, x)
            minY = min(minY, y); maxY = max(maxY, y)
        }
    }
    if maxX < 0 { print("\(url.lastPathComponent): empty mask"); failures += 1; continue }
    if trim {
        // CGContext rows run top-down here; Core Image's y axis runs bottom-up.
        let x0 = max(0, minX - pad), x1 = min(w, maxX + 1 + pad)
        let top = max(0, minY - pad), bottom = min(h, maxY + 1 + pad)
        let rect = CGRect(x: x0, y: h - bottom, width: x1 - x0, height: bottom - top)
        cut = cut.cropped(to: rect).transformed(by: CGAffineTransform(translationX: -rect.minX, y: -rect.minY))
    }
    if let maxSide, max(cut.extent.width, cut.extent.height) > CGFloat(maxSide) {
        let s = CGFloat(maxSide) / max(cut.extent.width, cut.extent.height)
        cut = cut.applyingFilter("CILanczosScaleTransform", parameters: [kCIInputScaleKey: s, kCIInputAspectRatioKey: 1.0])
    }
    // Two inputs with the same name (from different folders) must not
    // overwrite each other: disambiguate with -2, -3, ...
    var name = url.deletingPathExtension().lastPathComponent + ".png"
    var out = URL(fileURLWithPath: outDir).appendingPathComponent(name)
    var n = 2
    while fm.fileExists(atPath: out.path) {
        name = url.deletingPathExtension().lastPathComponent + "-\(n).png"
        out = URL(fileURLWithPath: outDir).appendingPathComponent(name)
        n += 1
    }
    do {
        try context.writePNGRepresentation(of: cut, to: out, format: .RGBA8, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)
    } catch {
        print("\(url.lastPathComponent): write failed: \(error.localizedDescription)"); failures += 1; continue
    }
    let share = Double(covered) / Double(w * h) * 100
    print(String(format: "%@ -> %@  %dx%d  subject %.0f%% of frame", url.lastPathComponent, name, Int(cut.extent.width), Int(cut.extent.height), share))
}
exit(failures == 0 ? 0 : 1)
