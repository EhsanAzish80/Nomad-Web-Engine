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
        
        DispatchQueue.global(qos: .utility).async { [weak self] in
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
    
    /// Navigates to a URL, resolving it relative to the current page.
    /// Use this for link clicks instead of loadURL.
    func navigate(_ urlString: String) {
        guard let engine = engine else {
            error = "Engine not initialized"
            return
        }
        
        isLoading = true
        error = nil
        
        DispatchQueue.global(qos: .utility).async { [weak self] in
            let result = urlString.withCString { urlPtr in
                nomad_engine_navigate(engine, urlPtr)
            }
            
            DispatchQueue.main.async {
                self?.isLoading = false
                
                if result == 0 {
                    self?.updateDisplayList()
                } else {
                    self?.error = "Failed to navigate (error code: \(result))"
                }
            }
        }
    }
    
    /// Updates the viewport width
    func setViewportWidth(_ width: Float) {
        guard let engine = engine else { return }
        DispatchQueue.global(qos: .utility).async { [weak self] in
            nomad_engine_set_viewport_width(engine, width)
            self?.updateDisplayList()
        }
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
        
        isLoading = true
        error = nil
        
        DispatchQueue.global(qos: .utility).async { [weak self] in
            // Convert inputs to JSON array format: [["key", "value"], ...]
            let inputsArray = inputs.map { [$0.0, $0.1] }
            
            guard let jsonData = try? JSONSerialization.data(withJSONObject: inputsArray),
                  let jsonString = String(data: jsonData, encoding: .utf8) else {
                self?.error = "Failed to encode form inputs"
                self?.isLoading = false
                return
            }
            
            let result = jsonString.withCString { jsonPtr in
                nomad_engine_submit_form(engine, formIndex, jsonPtr)
            }
            
            DispatchQueue.main.async {
                self?.isLoading = false
                
                if result == 0 {
                    self?.updateDisplayList()
                } else {
                    self?.error = "Failed to submit form (error code: \(result))"
                }
            }
        }
    }
    
    /// Goes back in navigation history
    func goBack() {
        guard let engine = engine else {
            error = "Engine not initialized"
            return
        }
        
        DispatchQueue.global(qos: .utility).async { [weak self] in
            let result = nomad_engine_go_back(engine)
            
            if result == 0 {
                self?.updateDisplayList()
            } else {
                DispatchQueue.main.async {
                    self?.error = "Cannot go back"
                }
            }
        }
    }
    
    /// Goes forward in navigation history
    func goForward() {
        guard let engine = engine else {
            error = "Engine not initialized"
            return
        }
        
        DispatchQueue.global(qos: .utility).async { [weak self] in
            let result = nomad_engine_go_forward(engine)
            
            if result == 0 {
                self?.updateDisplayList()
            } else {
                DispatchQueue.main.async {
                    self?.error = "Cannot go forward"
                }
            }
        }
    }
    
    /// Checks if can go back
    func canGoBack() -> Bool {
        guard let engine = engine else { return false }
        return nomad_engine_can_go_back(engine) == 1
    }
    
    /// Checks if can go forward
    func canGoForward() -> Bool {
        guard let engine = engine else { return false }
        return nomad_engine_can_go_forward(engine) == 1
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
