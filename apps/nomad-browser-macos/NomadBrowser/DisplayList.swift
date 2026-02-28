//
//  DisplayList.swift
//  NomadBrowser
//
//  Swift types for display list deserialization
//

import Foundation

/// A display list containing drawable items
struct DisplayList: Codable {
    let width: Float
    let height: Float
    let items: [DisplayItem]
}

/// A single drawable item
struct DisplayItem: Codable {
    let kind: DisplayItemKind
    let bounds: Rect
}

/// Type of display item
enum DisplayItemKind: Codable {
    case text(TextItem)
    
    private enum CodingKeys: String, CodingKey {
        case type
        case content
        case fontSize = "font_size"
        case isLink = "is_link"
        case linkUrl = "link_url"
    }
    
    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        
        // The Rust enum is serialized as a struct with "Text" variant
        let content = try container.decode(String.self, forKey: .content)
        let fontSize = try container.decode(Float.self, forKey: .fontSize)
        let isLink = try container.decode(Bool.self, forKey: .isLink)
        let linkUrl = try container.decodeIfPresent(String.self, forKey: .linkUrl)
        
        self = .text(TextItem(
            content: content,
            fontSize: fontSize,
            isLink: isLink,
            linkUrl: linkUrl
        ))
    }
    
    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        
        switch self {
        case .text(let item):
            try container.encode(item.content, forKey: .content)
            try container.encode(item.fontSize, forKey: .fontSize)
            try container.encode(item.isLink, forKey: .isLink)
            try container.encodeIfPresent(item.linkUrl, forKey: .linkUrl)
        }
    }
}

/// Text display item
struct TextItem: Codable {
    let content: String
    let fontSize: Float
    let isLink: Bool
    let linkUrl: String?
}

/// A rectangle
struct Rect: Codable {
    let x: Float
    let y: Float
    let width: Float
    let height: Float
    
    func contains(x: Float, y: Float) -> Bool {
        return x >= self.x && x <= self.x + self.width &&
               y >= self.y && y <= self.y + self.height
    }
}
