import Cocoa
import Foundation

// Test if Swift can load Google logo
let urlString = "https://www.google.com/images/branding/googlelogo/1x/googlelogo_white_background_color_272x92dp.png"

print("Testing image URL: \(urlString)")

let url = URL(string: urlString)!
let (data, response) = try await URLSession.shared.data(from: url)

print("Response: \(response)")
print("Data size: \(data.count) bytes")

if let nsImage = NSImage(data: data) {
    print("✓ Image loaded successfully!")
    print("  Size: \(nsImage.size.width) x \(nsImage.size.height)")
} else {
    print("✗ Failed to create NSImage from data")
}
