//
//  NomadEngine.swift
//  NomadBrowser
//
//  Swift wrapper around the Rust engine C API
//

import Foundation
import Combine

/// Wrapper around the Nomad Web Engine
class NomadEngineWrapper: ObservableObject {
    private var engine: OpaquePointer?
    
    @Published var displayList: DisplayList?
    @Published var isLoading: Bool = false
    @Published var error: String?
    
    init() {
        engine = nomad_engine_create()
        if engine == nil {
            error = "Failed to create engine"
        }
    }
    
    deinit {
        if let engine = engine {
            nomad_engine_destroy(engine)
        }
    }
    
    /// Loads a URL
    func loadURL(_ urlString: String) {
        guard let engine = engine else {
            error = "Engine not initialized"
            return
        }
        
        isLoading = true
        error = nil
        
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            let result = urlString.withCString { urlPtr in
                nomad_engine_load_url(engine, urlPtr)
            }
            
            DispatchQueue.main.async {
                self?.isLoading = false
                
                if result == 0 {
                    self?.updateDisplayList()
                } else {
                    self?.error = "Failed to load URL (error code: \(result))"
                }
            }
        }
    }
    
    /// Updates the viewport width
    func setViewportWidth(_ width: Float) {
        guard let engine = engine else { return }
        nomad_engine_set_viewport_width(engine, width)
        updateDisplayList()
    }
    
    /// Ticks the engine
    func tick() {
        guard let engine = engine else { return }
        nomad_engine_tick(engine)
    }
    
    /// Submits a form with the given inputs
    func submitForm(formIndex: Int, inputs: [(String, String)]) {
        guard let engine = engine else {
            error = "Engine not initialized"
            return
        }
        
        // TODO: Fix Swift bridging for nomad_engine_submit_form
        // The function exists in the dylib but Swift can't find it through the bridge
        // For now, manually construct the URL with query parameters
        print("Form submission - inputs: \(inputs)")
        
        // Build query string manually
        let queryString = inputs.map { name, value in
            "\(name.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? name)=\(value.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? value)"
        }.joined(separator: "&")
        
        // For now, just print it - full implementation requires fixing Swift FFI
        print("Would navigate to URL with query: ?\(queryString)")
        error = "Form submission not yet fully implemented in Swift layer"
    }
    
    /// Updates the display list from the engine
    private func updateDisplayList() {
        guard let engine = engine else { return }
        
        let buffer = nomad_engine_get_display_list(engine)
        
        guard buffer.data != nil, buffer.len > 0 else {
            error = "Empty display list"
            return
        }
        
        let data = Data(bytes: buffer.data!, count: buffer.len)
        nomad_free_byte_buffer(buffer)
        
        // Debug: print the JSON
        if let jsonString = String(data: data, encoding: .utf8) {
            print("Display list JSON: \(jsonString)")
        }
        
        do {
            // Decode JSON from the engine
            let decoder = JSONDecoder()
            let list = try decoder.decode(DisplayList.self, from: data)
            
            DispatchQueue.main.async {
                self.displayList = list
            }
        } catch {
            DispatchQueue.main.async {
                self.error = "Failed to decode display list: \(error.localizedDescription)"
            }
        }
    }
}
