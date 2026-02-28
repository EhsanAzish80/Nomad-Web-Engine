//
//  RenderView.swift
//  NomadBrowser
//
//  Custom view for rendering the display list
//

import SwiftUI
import AppKit

struct RenderView: NSViewRepresentable {
    let displayList: DisplayList
    let onLinkClick: (String) -> Void
    
    func makeNSView(context: Context) -> RenderNSView {
        let view = RenderNSView()
        view.onLinkClick = onLinkClick
        return view
    }
    
    func updateNSView(_ nsView: RenderNSView, context: Context) {
        nsView.displayList = displayList
        nsView.needsDisplay = true
    }
}

class RenderNSView: NSView {
    var displayList: DisplayList?
    var onLinkClick: ((String) -> Void)?
    
    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        setupTracking()
    }
    
    required init?(coder: NSCoder) {
        super.init(coder: coder)
        setupTracking()
    }
    
    private func setupTracking() {
        // Add tracking area for mouse events
        let trackingArea = NSTrackingArea(
            rect: bounds,
            options: [.activeInKeyWindow, .inVisibleRect, .mouseMoved],
            owner: self,
            userInfo: nil
        )
        addTrackingArea(trackingArea)
    }
    
    override func draw(_ dirtyRect: NSRect) {
        super.draw(dirtyRect)
        
        guard let context = NSGraphicsContext.current?.cgContext,
              let displayList = displayList else {
            return
        }
        
        // Fill background
        context.setFillColor(NSColor.white.cgColor)
        context.fill(bounds)
        
        // Draw all items
        for item in displayList.items {
            switch item.kind {
            case .text(let textItem):
                drawText(textItem, bounds: item.bounds, in: context)
            }
        }
    }
    
    private func drawText(_ textItem: TextItem, bounds: Rect, in context: CGContext) {
        let rect = CGRect(
            x: CGFloat(bounds.x),
            y: CGFloat(bounds.y),
            width: CGFloat(bounds.width),
            height: CGFloat(bounds.height)
        )
        
        // Set text color
        let textColor = textItem.isLink ? NSColor.blue : NSColor.black
        
        // Create attributed string
        let attributes: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: CGFloat(textItem.fontSize)),
            .foregroundColor: textColor,
            .underlineStyle: textItem.isLink ? NSUnderlineStyle.single.rawValue : 0
        ]
        
        let attributedString = NSAttributedString(
            string: textItem.content,
            attributes: attributes
        )
        
        // Draw text
        attributedString.draw(at: rect.origin)
    }
    
    override func mouseDown(with event: NSEvent) {
        guard let displayList = displayList else { return }
        
        let location = convert(event.locationInWindow, from: nil)
        let flippedY = bounds.height - location.y // Flip Y coordinate
        
        // Check if click hits a link
        for item in displayList.items {
            if case .text(let textItem) = item.kind,
               textItem.isLink,
               let url = textItem.linkUrl,
               item.bounds.contains(x: Float(location.x), y: Float(flippedY)) {
                
                onLinkClick?(url)
                return
            }
        }
    }
    
    override var isFlipped: Bool {
        return true // Use top-left origin like web coordinates
    }
}
