# Testing Guide for Milestone 0

This guide helps you verify that Milestone 0 is working correctly.

## Prerequisites

Ensure Rust is installed:
```bash
rustc --version
cargo --version
```

## Building the Project

```bash
# From the project root
cd /path/to/nomad-web-engine

# Build all crates
cargo build
```

Expected output: All crates should build without errors.

## Running Tests

### All Tests

```bash
cargo test
```

Expected: All tests pass.

### Per-Crate Tests

```bash
# Test network layer
cargo test -p nomad-net

# Test HTML parser
cargo test -p nomad-html

# Test core engine
cargo test -p nomad-core

# Test C API
cargo test -p nomad-c-api
```

## Manual Testing

### Test 1: Simple Website

```bash
cargo run --example headless-demo -- https://example.com
```

**Expected Output**:
```
Loading: https://example.com
---
Example Domain This domain is for use in illustrative examples...
```

### Test 2: Wikipedia Page

```bash
cargo run --example headless-demo -- https://en.wikipedia.org/wiki/Rust_(programming_language)
```

**Expected**: Should output the article text without any script/style content.

### Test 3: News Site

```bash
cargo run --example headless-demo -- https://www.bbc.com
```

**Expected**: Should output news article text.

### Test 4: Invalid URL

```bash
cargo run --example headless-demo -- not-a-url
```

**Expected**: Error message about invalid URL.

### Test 5: Unsupported Protocol

```bash
cargo run --example headless-demo -- ftp://example.com
```

**Expected**: Error message about unsupported scheme.

### Test 6: Non-existent Domain

```bash
cargo run --example headless-demo -- https://this-domain-does-not-exist-12345.com
```

**Expected**: Network error (DNS failure or connection timeout).

## Testing Custom Configurations

Create a test file `test_config.rs`:

```rust
use nomad_core::{Engine, EngineConfig};

fn main() {
    // Very restrictive limits
    let config = EngineConfig {
        max_html_size: 1024,      // 1 KB
        max_dom_nodes: 100,       // 100 nodes
        timeout_secs: 5,          // 5 seconds
    };
    
    let engine = Engine::with_config(config).unwrap();
    
    // This should fail due to size limit
    match engine.load_url("https://www.wikipedia.org") {
        Ok(_) => println!("Unexpectedly succeeded"),
        Err(e) => println!("Expected error: {}", e),
    }
}
```

Run:
```bash
rustc test_config.rs --edition 2021 -L target/debug/deps
./test_config
```

## Verification Checklist

After running the tests above, verify:

- [ ] All unit tests pass
- [ ] Can fetch and parse simple HTML (example.com)
- [ ] Can fetch and parse complex HTML (Wikipedia)
- [ ] Text extraction works correctly
- [ ] Script/style tags are filtered out
- [ ] Invalid URLs are rejected
- [ ] Unsupported protocols are rejected
- [ ] Network errors are handled gracefully
- [ ] Size limits are enforced
- [ ] Timeout limits work
- [ ] Custom configuration works

## Performance Testing

### Measure Parse Time

Add timing to the demo:

```rust
use std::time::Instant;

let start = Instant::now();
let text = engine.load_url(url)?;
let elapsed = start.elapsed();

println!("Time: {:?}", elapsed);
println!("Text length: {} bytes", text.len());
```

### Expected Performance

On a typical machine:
- Simple page (example.com): < 1 second
- Medium page (Wikipedia article): 1-3 seconds
- Large page: 3-10 seconds

Note: Times include network latency.

## Memory Testing

### Check Memory Usage

```bash
# On macOS/Linux
/usr/bin/time -l cargo run --example headless-demo -- https://example.com

# Look for "maximum resident set size"
```

### Expected Memory

- Base overhead: ~10-20 MB
- Per KB of HTML: ~2-5 KB additional
- Per 1000 DOM nodes: ~100-200 KB additional

## Edge Cases

### Test Malformed HTML

Create a test file with broken HTML:

```html
<!DOCTYPE html>
<html>
<body>
<p>Unclosed paragraph
<div>Unclosed div
<span>Some text
</body>
```

Save as `malformed.html` and serve with a local server, or test directly:

```rust
use nomad_html::HtmlParser;

let html = r#"
<p>Unclosed paragraph
<div>Unclosed div
<span>Some text
"#;

let parser = HtmlParser::new();
let dom = parser.parse(html).unwrap();
let text = dom.extract_text();
println!("{}", text);
```

**Expected**: Should parse successfully (html5ever handles malformed HTML).

### Test Empty HTML

```rust
let parser = HtmlParser::new();
let dom = parser.parse("").unwrap();
assert_eq!(dom.extract_text(), "");
```

### Test HTML with Only Scripts

```rust
let html = "<html><script>alert('test');</script></html>";
let parser = HtmlParser::new();
let dom = parser.parse(html).unwrap();
assert_eq!(dom.extract_text().trim(), "");
```

### Test Very Large DOM

```rust
use nomad_html::HtmlParser;

// Generate HTML with many nodes
let mut html = String::from("<html><body>");
for i in 0..10000 {
    html.push_str(&format!("<p>Paragraph {}</p>", i));
}
html.push_str("</body></html>");

let parser = HtmlParser::with_max_nodes(50000);
let result = parser.parse(&html);

assert!(result.is_ok());
```

## Debugging Tips

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run --example headless-demo -- https://example.com
```

### Run with Backtrace

```bash
RUST_BACKTRACE=1 cargo run --example headless-demo -- https://example.com
```

### Check Individual Components

```bash
# Test just the network layer
cargo test -p nomad-net -- --nocapture

# Test just the HTML parser
cargo test -p nomad-html -- --nocapture
```

## CI/CD Testing

The GitHub Actions workflow (`.github/workflows/ci.yml`) runs:

1. Format check: `cargo fmt --all -- --check`
2. Lint: `cargo clippy --all-targets --all-features`
3. Check: `cargo check --all-targets --all-features`
4. Build: `cargo build`
5. Test: `cargo test --all`
6. Release build: `cargo build --release`

On three platforms:
- Ubuntu (Linux)
- macOS
- Windows

## Common Issues

### Issue: "cargo: command not found"

**Solution**: Install Rust via rustup: https://rustup.rs/

### Issue: Build fails with dependency errors

**Solution**:
```bash
cargo clean
cargo update
cargo build
```

### Issue: Tests timeout

**Solution**: Increase test timeout or check network connection.

### Issue: "Response too large"

**Solution**: Increase `max_html_size` in configuration or skip large sites.

### Issue: SSL/TLS errors

**Solution**: Update system certificates or check that `ca-certificates` is installed.

## Success Criteria

Milestone 0 is successfully implemented if:

1. ✅ All unit tests pass
2. ✅ Can fetch HTTP/HTTPS pages
3. ✅ HTML5 parsing works with malformed HTML
4. ✅ Text extraction filters non-visible elements
5. ✅ Resource limits are enforced
6. ✅ Errors are handled gracefully
7. ✅ Headless demo runs on real websites
8. ✅ Code compiles on Linux, macOS, and Windows

## Next Steps

Once testing is complete:
1. Review [MILESTONE_0.md](MILESTONE_0.md) for implementation details
2. Check [QUICKSTART.md](QUICKSTART.md) for usage examples
3. See [ARCHITECTURE.md](ARCHITECTURE.md) for design decisions
4. Read [CONTRIBUTING.md](CONTRIBUTING.md) to contribute

## Reporting Issues

If you find bugs:
1. Check if the issue is already reported
2. Provide minimal reproduction steps
3. Include Rust version (`rustc --version`)
4. Include OS and version
5. Include error messages and backtraces

Happy testing! 🚀
