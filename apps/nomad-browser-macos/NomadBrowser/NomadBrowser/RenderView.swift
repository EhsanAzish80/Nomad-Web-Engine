//
//  RenderView.swift
//  NomadBrowser
//
//  Custom view for rendering the display list
//

import SwiftUI
import AppKit

struct RenderView: View {
    let displayList: DisplayList
    let onLinkClick: (String) -> Void
    let onFormSubmit: ([(String, String)]) -> Void
    @State private var inputValues: [String: String] = [:]
    @State private var loadedImages: [String: NSImage] = [:]
    
    var body: some View {
        ZStack(alignment: .topLeading) {
            // Canvas for text and image rendering
            // Note: Canvas re-renders when loadedImages changes because we access it in the closure
            Canvas { context, size in
                for item in displayList.items {
                    if case .text(let textItem) = item.kind {
                        drawText(textItem, bounds: item.bounds, in: &context)
                    } else if case .image(let imageItem) = item.kind {
                        drawImage(imageItem, bounds: item.bounds, in: &context)
                    }
                }
            }
            .background(Color.white)
            // Force Canvas to re-draw when images load
            .id(loadedImages.count)
            
            // Overlay interactive elements
            ForEach(displayList.items.indices, id: \.self) { index in
                let item = displayList.items[index]
                
                switch item.kind {
                case .input(let inputItem):
                    TextField(inputItem.value.isEmpty ? "Enter \(inputItem.name)" : inputItem.value,
                              text: Binding(
                                get: { inputValues[inputItem.name] ?? inputItem.value },
                                set: { inputValues[inputItem.name] = $0 }
                              ))
                    .textFieldStyle(.roundedBorder)
                    .frame(width: max(1, CGFloat(item.bounds.width - 4)), height: max(1, CGFloat(item.bounds.height - 4)))
                    .position(x: CGFloat(item.bounds.x + item.bounds.width / 2),
                             y: CGFloat(item.bounds.y + item.bounds.height / 2))
                    
                case .button(let buttonItem):
                    Button(buttonItem.label) {
                        handleButtonClick(buttonItem)
                    }
                    .buttonStyle(.borderedProminent)
                    .frame(width: max(1, CGFloat(item.bounds.width - 4)), height: max(1, CGFloat(item.bounds.height - 4)))
                    .position(x: CGFloat(item.bounds.x + item.bounds.width / 2),
                             y: CGFloat(item.bounds.y + item.bounds.height / 2))
                    
                case .link(let linkItem):
                    Button("") {
                        onLinkClick(linkItem.url)
                    }
                    .buttonStyle(.plain)
                    .frame(width: max(1, CGFloat(item.bounds.width)), height: max(1, CGFloat(item.bounds.height)))
                    .position(x: CGFloat(item.bounds.x + item.bounds.width / 2),
                             y: CGFloat(item.bounds.y + item.bounds.height / 2))
                    .opacity(0.0) // Invisible button for click handling
                    
                case .text(let textItem):
                    if textItem.isLink, let url = textItem.linkUrl {
                        Button("") {
                            onLinkClick(url)
                        }
                        .buttonStyle(.plain)
                        .frame(width: max(1, CGFloat(item.bounds.width)), height: max(1, CGFloat(item.bounds.height)))
                        .position(x: CGFloat(item.bounds.x + item.bounds.width / 2),
                                 y: CGFloat(item.bounds.y + item.bounds.height / 2))
                        .opacity(0.0) // Invisible button over text for click handling
                    } else {
                        EmptyView()
                    }
                
                case .image:
                    // Images are non-interactive, rendered in Canvas
                    EmptyView()
                }
            }
        }
        .onAppear {
            // Initialize input values from display list
            for item in displayList.items {
                if case .input(let inputItem) = item.kind {
                    inputValues[inputItem.name] = inputItem.value
                } else if case .image(let imageItem) = item.kind {
                    // Load image asynchronously
                    loadImage(from: imageItem.src)
                }
            }
        }
    }
    
    private func drawText(_ textItem: TextItem, bounds: Rect, in context: inout GraphicsContext) {
        let textColor = textItem.isLink ? Color.blue : Color.black
        
        var attributedString = AttributedString(textItem.content)
        attributedString.font = .system(size: CGFloat(textItem.fontSize))
        attributedString.foregroundColor = textColor
        if textItem.isLink {
            attributedString.underlineStyle = .single
        }
        
        // Apply text alignment
        let (anchor, xOffset): (UnitPoint, CGFloat) = {
            switch textItem.textAlign {
            case .Left:
                return (.topLeading, 0)
            case .Center:
                return (.top, CGFloat(bounds.width) / 2)
            case .Right:
                return (.topTrailing, CGFloat(bounds.width))
            }
        }()
        
        let drawPoint = CGPoint(x: CGFloat(bounds.x) + xOffset, y: CGFloat(bounds.y))
        context.draw(Text(attributedString), at: drawPoint, anchor: anchor)
    }
    
    private func drawImage(_ imageItem: ImageItem, bounds: Rect, in context: inout GraphicsContext) {
        guard let nsImage = loadedImages[imageItem.src] else {
            // Image not loaded yet, draw placeholder
            let rect = CGRect(
                x: CGFloat(bounds.x),
                y: CGFloat(bounds.y),
                width: CGFloat(bounds.width),
                height: CGFloat(bounds.height)
            )
            
            // Draw gray border
            var path = Path()
            path.addRect(rect)
            context.stroke(path, with: .color(.gray), lineWidth: 1)
            
            // Draw "Loading..." text or alt text
            let placeholder = imageItem.alt.isEmpty ? "Loading..." : imageItem.alt
            var attributedString = AttributedString(placeholder)
            attributedString.font = .system(size: 10)
            attributedString.foregroundColor = .gray
            
            let center = CGPoint(x: CGFloat(bounds.x) + CGFloat(bounds.width) / 2,
                                y: CGFloat(bounds.y) + CGFloat(bounds.height) / 2)
            context.draw(Text(attributedString), at: center, anchor: .center)
            return
        }
        
        // Draw the loaded image
        let rect = CGRect(
            x: CGFloat(bounds.x),
            y: CGFloat(bounds.y),
            width: CGFloat(bounds.width),
            height: CGFloat(bounds.height)
        )
        
        if let cgImage = nsImage.cgImage(forProposedRect: nil, context: nil, hints: nil) {
            let image = Image(decorative: cgImage, scale: 1.0)
            context.draw(image, in: rect)
        }
    }
    
    private func handleButtonClick(_ button: ButtonItem) {
        if button.buttonType == "submit" {
            // Collect all input values and submit form
            let inputs = inputValues.map { (key, value) in (key, value) }
            onFormSubmit(inputs)
        }
    }
    
    private func loadImage(from urlString: String) {
        // Don't reload if already loaded
        guard loadedImages[urlString] == nil else { return }
        
        guard let url = URL(string: urlString) else { return }
        
        // Fetch image asynchronously
        Task {
            do {
                let (data, _) = try await URLSession.shared.data(from: url)
                if let nsImage = NSImage(data: data) {
                    await MainActor.run {
                        loadedImages[urlString] = nsImage
                    }
                }
            } catch {
                print("Failed to load image from \(urlString): \(error)")
            }
        }
    }
}
