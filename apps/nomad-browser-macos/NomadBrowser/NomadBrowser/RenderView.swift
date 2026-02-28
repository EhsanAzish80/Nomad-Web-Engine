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
    
    var body: some View {
        ZStack(alignment: .topLeading) {
            // Canvas for text rendering
            Canvas { context, size in
                for item in displayList.items {
                    if case .text(let textItem) = item.kind {
                        drawText(textItem, bounds: item.bounds, in: &context)
                    }
                }
            }
            .background(Color.white)
            
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
                    .frame(width: CGFloat(item.bounds.width - 4), height: CGFloat(item.bounds.height - 4))
                    .position(x: CGFloat(item.bounds.x + item.bounds.width / 2),
                             y: CGFloat(item.bounds.y + item.bounds.height / 2))
                    
                case .button(let buttonItem):
                    Button(buttonItem.label) {
                        handleButtonClick(buttonItem)
                    }
                    .buttonStyle(.borderedProminent)
                    .frame(width: CGFloat(item.bounds.width - 4), height: CGFloat(item.bounds.height - 4))
                    .position(x: CGFloat(item.bounds.x + item.bounds.width / 2),
                             y: CGFloat(item.bounds.y + item.bounds.height / 2))
                    
                case .link(let linkItem):
                    Button("") {
                        onLinkClick(linkItem.url)
                    }
                    .buttonStyle(.plain)
                    .frame(width: CGFloat(item.bounds.width), height: CGFloat(item.bounds.height))
                    .position(x: CGFloat(item.bounds.x + item.bounds.width / 2),
                             y: CGFloat(item.bounds.y + item.bounds.height / 2))
                    .opacity(0.0) // Invisible button for click handling
                    
                case .text(let textItem):
                    if textItem.isLink, let url = textItem.linkUrl {
                        Button("") {
                            onLinkClick(url)
                        }
                        .buttonStyle(.plain)
                        .frame(width: CGFloat(item.bounds.width), height: CGFloat(item.bounds.height))
                        .position(x: CGFloat(item.bounds.x + item.bounds.width / 2),
                                 y: CGFloat(item.bounds.y + item.bounds.height / 2))
                        .opacity(0.0) // Invisible button over text for click handling
                    }
                }
            }
        }
        .onAppear {
            // Initialize input values from display list
            for item in displayList.items {
                if case .input(let inputItem) = item.kind {
                    inputValues[inputItem.name] = inputItem.value
                }
            }
        }
    }
    
    private func drawText(_ textItem: TextItem, bounds: Rect, in context: inout GraphicsContext) {
        let rect = CGRect(
            x: CGFloat(bounds.x),
            y: CGFloat(bounds.y),
            width: CGFloat(bounds.width),
            height: CGFloat(bounds.height)
        )
        
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
    
    private func handleButtonClick(_ button: ButtonItem) {
        if button.buttonType == "submit" {
            // Collect all input values and submit form
            let inputs = inputValues.map { (key, value) in (key, value) }
            onFormSubmit(inputs)
        }
    }
}
