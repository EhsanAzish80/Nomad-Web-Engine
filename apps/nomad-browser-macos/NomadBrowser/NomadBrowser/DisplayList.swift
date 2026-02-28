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
    let hitRegions: [HitRegion]
    
    enum CodingKeys: String, CodingKey {
        case width
        case height
        case items
        case hitRegions = "hit_regions"
    }
}

/// A single drawable item
struct DisplayItem: Codable {
    let kind: DisplayItemKind
    let bounds: Rect
}

/// Type of display item
enum DisplayItemKind: Codable {
    case text(TextItem)
    case link(LinkItem)
    case input(InputItem)
    case button(ButtonItem)
    case image(ImageItem)
    
    private enum CodingKeys: String, CodingKey {
        case Text
        case Link
        case Input
        case Button
        case Image
    }
    
    init(from decoder: Decoder) throws {
        // Rust serializes enums as {"VariantName": {fields}}
        let container = try decoder.container(keyedBy: CodingKeys.self)
        
        if let textData = try? container.decode(TextItem.self, forKey: .Text) {
            self = .text(textData)
        } else if let linkData = try? container.decode(LinkItem.self, forKey: .Link) {
            self = .link(linkData)
        } else if let inputData = try? container.decode(InputItem.self, forKey: .Input) {
            self = .input(inputData)
        } else if let buttonData = try? container.decode(ButtonItem.self, forKey: .Button) {
            self = .button(buttonData)
        } else if let imageData = try? container.decode(ImageItem.self, forKey: .Image) {
            self = .image(imageData)
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
        case .link(let item):
            try container.encode(item, forKey: .Link)
        case .input(let item):
            try container.encode(item, forKey: .Input)
        case .button(let item):
            try container.encode(item, forKey: .Button)
        case .image(let item):
            try container.encode(item, forKey: .Image)
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

/// Link display item
struct LinkItem: Codable {
    let url: String
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

/// Image display item
struct ImageItem: Codable {
    let src: String
    let alt: String
    let width: Float
    let height: Float
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

/// An interactive hit region
struct HitRegion: Codable {
    let id: String
    let kind: HitRegionKind
    let rect: Rect
}

/// Type of hit region
enum HitRegionKind: Codable {
    case link(HitRegionLink)
    case button(HitRegionButton)
    case input(HitRegionInput)
    
    private enum CodingKeys: String, CodingKey {
        case Link
        case Button
        case Input
    }
    
    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        
        if let linkData = try? container.decode(HitRegionLink.self, forKey: .Link) {
            self = .link(linkData)
        } else if let buttonData = try? container.decode(HitRegionButton.self, forKey: .Button) {
            self = .button(buttonData)
        } else if let inputData = try? container.decode(HitRegionInput.self, forKey: .Input) {
            self = .input(inputData)
        } else {
            throw DecodingError.dataCorrupted(
                DecodingError.Context(
                    codingPath: decoder.codingPath,
                    debugDescription: "Unknown hit region kind"
                )
            )
        }
    }
    
    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        
        switch self {
        case .link(let data):
            try container.encode(data, forKey: .Link)
        case .button(let data):
            try container.encode(data, forKey: .Button)
        case .input(let data):
            try container.encode(data, forKey: .Input)
        }
    }
}

/// Link hit region data
struct HitRegionLink: Codable {
    let url: String
}

/// Button hit region data
struct HitRegionButton: Codable {
    let buttonType: String
    let formId: String?
    
    enum CodingKeys: String, CodingKey {
        case buttonType = "button_type"
        case formId = "form_id"
    }
}

/// Input hit region data
struct HitRegionInput: Codable {
    let controlId: String
    let name: String
    let inputType: String
    
    enum CodingKeys: String, CodingKey {
        case controlId = "control_id"
        case name
        case inputType = "input_type"
    }
}
