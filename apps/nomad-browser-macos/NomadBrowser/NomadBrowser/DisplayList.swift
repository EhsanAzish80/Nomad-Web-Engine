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
        case Text
    }
    
    init(from decoder: Decoder) throws {
        // Rust serializes enums as {"VariantName": {fields}}
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let textData = try container.decode(TextItem.self, forKey: .Text)
        self = .text(textData)
    }
    
    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        
        switch self {
        case .text(let item):
            try container.encode(item, forKey: .Text)
        }
    }
}

/// Text display item
struct TextItem: Codable {
    let content: String
    let fontSize: Float
    let isLink: Bool
    let linkUrl: String?
    
    enum CodingKeys: String, CodingKey {
        case content
        case fontSize = "font_size"
        case isLink = "is_link"
        case linkUrl = "link_url"
    }
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
