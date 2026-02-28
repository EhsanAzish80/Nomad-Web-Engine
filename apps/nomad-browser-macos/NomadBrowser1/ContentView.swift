//
//  ContentView.swift
//  NomadBrowser
//
//  Main browser window view
//

import SwiftUI

struct ContentView: View {
    @StateObject private var engine = NomadEngineWrapper()
    @State private var urlText: String = "https://example.com"
    @State private var viewportWidth: CGFloat = 800
    
    var body: some View {
        VStack(spacing: 0) {
            // URL bar
            HStack {
                TextField("Enter URL", text: $urlText)
                    .textFieldStyle(.roundedBorder)
                    .onSubmit {
                        loadURL()
                    }
                
                Button("Go") {
                    loadURL()
                }
                .buttonStyle(.borderedProminent)
            }
            .padding()
            
            Divider()
            
            // Error message
            if let error = engine.error {
                Text(error)
                    .foregroundColor(.red)
                    .padding()
                    .frame(maxWidth: .infinity)
                    .background(Color.red.opacity(0.1))
            }
            
            // Loading indicator
            if engine.isLoading {
                ProgressView()
                    .padding()
            }
            
            // Render view
            if let displayList = engine.displayList {
                ScrollView(.vertical) {
                    RenderView(
                        displayList: displayList,
                        onLinkClick: { url in
                            urlText = url
                            loadURL()
                        }
                    )
                    .frame(width: viewportWidth, height: displayList.height)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if !engine.isLoading {
                Text("Enter a URL to browse")
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .frame(minWidth: 600, minHeight: 400)
        .onAppear {
            engine.setViewportWidth(Float(viewportWidth))
        }
    }
    
    private func loadURL() {
        // Ensure URL has a scheme
        var url = urlText.trimmingCharacters(in: .whitespaces)
        if !url.hasPrefix("http://") && !url.hasPrefix("https://") {
            url = "https://" + url
        }
        engine.loadURL(url)
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
