# Headless Demo

A simple command-line application that demonstrates the Nomad Web Engine's text extraction capabilities.

## Overview

This example loads a URL, fetches the HTML, parses it, and extracts the visible text content, printing it to stdout.

## Features

- HTTP/HTTPS GET requests with automatic redirect following
- HTML5-compliant parsing that tolerates malformed HTML
- Text extraction from visible elements only (excludes scripts, styles, etc.)
- Built-in safety limits:
  - Max response size: 10 MB
  - Max DOM nodes: 100,000
  - Request timeout: 30 seconds

## Usage

```bash
# Build the example
cargo build --example headless-demo

# Run the example
cargo run --example headless-demo -- <url>

# Example
cargo run --example headless-demo -- https://example.com
```

## What It Demonstrates

1. **Network Layer** (`nomad-net`):
   - Fetches HTML from URLs
   - Handles HTTPS certificates
   - Follows redirects automatically
   - Enforces timeouts and size limits

2. **HTML Parser** (`nomad-html`):
   - Parses HTML5 documents
   - Tolerates malformed HTML gracefully
   - Builds internal DOM structure
   - Limits DOM tree size

3. **Engine** (`nomad-core`):
   - Orchestrates network and parsing
   - Extracts visible text from DOM
   - Cleans and normalizes whitespace

## Example Output

```bash
$ cargo run --example headless-demo -- https://example.com

Loading: https://example.com
---
Example Domain This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission. More information...
```

## Architecture

```
User Input (URL)
    ↓
[Engine.load_url()]
    ↓
[NetworkLayer.fetch()] ← HTTP/HTTPS GET
    ↓
[HtmlParser.parse()] ← HTML5 parsing
    ↓
[DomTree.extract_text()] ← Text extraction
    ↓
Output to stdout
```

## Limitations (Milestone 0)

This is a minimal reader implementation:

- ❌ No CSS parsing or styling
- ❌ No layout computation
- ❌ No JavaScript execution
- ❌ No image loading
- ❌ No rendering or display
- ✅ HTML parsing
- ✅ Text extraction
- ✅ Network fetching

## Next Steps

Future milestones will add:
- CSS parsing and computed styles
- Layout engine (box model)
- JavaScript execution
- Rendering to screen
