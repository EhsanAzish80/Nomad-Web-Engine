//
//  NomadBrowserApp.swift
//  NomadBrowser
//
//  Nomad Web Engine macOS Browser
//

import SwiftUI

@main
struct NomadBrowserApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
        .windowStyle(.automatic)
        .windowResizability(.contentSize)
        .commands {
            CommandGroup(replacing: .newItem) {}
        }
    }
}
