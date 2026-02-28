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
    case input(InputItem)
    case button(ButtonItem)
    
    private enum CodingKeys: String, CodingKey {
        case Text
        case Input
        case Button
    }
    
    init(from decoder: Decoder) throws {
        // Rust serializes enums as {"VariantName": {fields}}
        let container = try decoder.container(keyedBy: CodingKeys.self)
        
        if let textData = try? container.decode(TextItem.self, forKey: .Text) {
            self = .text(textData)
        } else if let inputData = try? container.decode(InputItem.self, forKey: .Input) {
            self = .input(inputData)
        } else if let buttonData = try? container.decode(ButtonItem.self, forKey: .Button) {
            self = .button(buttonData)
        } else {
            throw DecodingError.dataCorrupted(
                DecodingError.Context(
                    codingPath: decoder.codingPath,
                    debugDescription: "Unknown display item kind"
                )
            )
        }
    }
    
    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        
        switch self {
        case .text(let item):
            try container.encode(item, forKey: .Text)
        case .input(let item):
            try container.encode(item, forKey: .Input)
        case .button(let item):
            try container.encode(item, forKey: .Button)
        }
    }
}

/// Text display item
struct TextItem: Codable {
    let content: String
    let fontSize: Float
    let isLink: Bool
    let linkUrl: String?
    let textAlign: TextAlign
    
    enum CodingKeys: String, CodingKey {
        case content
        case fontSize = "font_size"
        case isLink = "is_link"
        case linkUrl = "link_url"
        case textAlign = "text_align"
    }
}

/// Text alignment
enum TextAlign: String, Codable {
    case Left
    case Center
    case Right
}

/// Input display item
struct InputItem: Codable {
    let name: String
    let value: String
    let inputType: String
    
    enum CodingKeys: String, CodingKey {
        case name
        case value
        case inputType = "input_type"
    }
}

/// Button display item
struct ButtonItem: Codable {
    let label: String
    let buttonType: String
    let formId: String?
    
    enum CodingKeys: String, CodingKey {
        case label
        case buttonType = "button_type"
        case formId = "form_id"
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
