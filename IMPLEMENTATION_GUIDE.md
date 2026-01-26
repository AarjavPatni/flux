# Image Processor - Implementation Guide

## Session 1: Build Pieces 1-3 (Estimated: 1 hour)

---

## **Piece 1: URL Generator** (5 min)

### What & Why
Generate Lorem Picsum URLs. Simple utility to get test images without dealing with datasets.

### Your Task
```rust
// src/url_generator.rs

pub struct UrlGenerator {
    count: usize,
}

impl UrlGenerator {
    pub fn new(count: usize) -> Self {
        todo!()
    }
    
    /// Generate URLs for random images from Lorem Picsum
    /// Format: https://picsum.photos/seed/{i}/800/600
    /// Using seed ensures same images across runs
    pub fn generate(&self) -> Vec<String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn generates_correct_count() {
        let gen = UrlGenerator::new(10);
        let urls = gen.generate();
        assert_eq!(urls.len(), 10);
    }
    
    #[test]
    fn urls_have_correct_format() {
        let gen = UrlGenerator::new(5);
        let urls = gen.generate();
        assert!(urls[0].contains("picsum.photos"));
        assert!(urls[0].contains("/800/600"));
    }
}
```

### Acceptance
- `cargo test url_generator` passes
- Prints 10 URLs when you run it

---

## **Piece 2: Single Image Processor** (30 min)

### What & Why
The atomic unit: download → decode → resize → save ONE image.
Everything else is just orchestrating multiple calls with different concurrency.

### Your Task
```rust
// src/image_processor.rs

use anyhow::Result;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ImageMetrics {
    pub url: String,
    pub download_ms: u64,
    pub decode_ms: u64,
    pub resize_ms: u64,
    pub save_ms: u64,
    pub bytes_downloaded: usize,
}

/// Process a single image: download → decode → resize → save
pub async fn process_single_image(
    url: &str,
    output_dir: &Path,
) -> Result<ImageMetrics> {
    todo!("Implement this!")
    
    // Steps you need:
    // 1. Download bytes with reqwest::get(url).await?.bytes().await?
    // 2. Decode with image::load_from_memory(&bytes)?
    // 3. Resize with img.resize(256, 256, image::imageops::FilterType::Lanczos3)
    // 4. Save with resized.save(output_dir.join(filename))?
    // 5. Track timing for each step (see pattern below)
    // 6. Generate filename from URL hash or just use image_{timestamp}.jpg
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[tokio::test]
    async fn processes_single_image() {
        let output = Path::new("test_output");
        fs::create_dir_all(output).unwrap();
        
        let url = "https://picsum.photos/seed/1/800/600";
        let result = process_single_image(url, output).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.download_ms > 0);
        assert!(metrics.bytes_downloaded > 0);
        
        // Cleanup
        fs::remove_dir_all(output).unwrap();
    }
}
```

### Key Patterns

**Pattern 1: Timing Operations**
```rust
use std::time::Instant;

let start = Instant::now();
let result = some_async_operation().await?;
let elapsed_ms = start.elapsed().as_millis() as u64;
```

**Pattern 2: reqwest Async Download**
```rust
let response = reqwest::get(url).await?;
let bytes = response.bytes().await?;
let byte_count = bytes.len();
```

**Pattern 3: Image Decode & Resize**
```rust
use image::imageops::FilterType;

let img = image::load_from_memory(&bytes)?;
let resized = img.resize(256, 256, FilterType::Lanczos3);
```

**Pattern 4: Generate Filename**
```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash_url(url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// Then: output_dir.join(format!("{}.jpg", hash_url(url)))
```

### Acceptance
- `cargo test image_processor` passes
- Run manually: processes one image, creates file in output dir
- Metrics show reasonable timing (download > 100ms, resize < 500ms)

---

## **Piece 3: Memory Monitor** (20 min)

### What & Why
Track memory usage in real-time. Critical for seeing the OOM problem in naive approach.

### Your Task
```rust
// src/memory_monitor.rs

use sysinfo::System;

pub struct MemoryMonitor {
    system: System,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        todo!()
    }
    
    /// Get current memory usage in MB
    pub fn current_usage_mb(&mut self) -> u64 {
        todo!()
        // Hint: self.system.refresh_memory();
        // Then: self.system.used_memory() / 1_024 / 1_024
    }
    
    /// Get available memory in MB
    pub fn available_mb(&mut self) -> u64 {
        todo!()
    }
    
    /// Get memory usage as percentage (0-100)
    pub fn usage_percent(&mut self) -> f32 {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn reports_memory() {
        let mut monitor = MemoryMonitor::new();
        let usage = monitor.current_usage_mb();
        assert!(usage > 0);
        assert!(usage < 1_000_000); // Less than 1TB :)
    }
    
    #[test]
    fn reports_percentage() {
        let mut monitor = MemoryMonitor::new();
        let percent = monitor.usage_percent();
        assert!(percent > 0.0);
        assert!(percent <= 100.0);
    }
}
```

### Key Pattern: sysinfo Usage
```rust
use sysinfo::System;

let mut system = System::new_all();
system.refresh_memory();

let used = system.used_memory(); // bytes
let total = system.total_memory(); // bytes
let percent = (used as f64 / total as f64) * 100.0;
```

### Acceptance
- `cargo test memory_monitor` passes
- Prints current memory when run
- Shows percentage that makes sense for your machine

---

## **Quick Start Commands**

```bash
# Create module files
touch src/url_generator.rs
touch src/image_processor.rs
touch src/memory_monitor.rs

# Update main.rs to declare modules
echo "mod url_generator;
mod image_processor;
mod memory_monitor;

fn main() {
    println!(\"Image Processor - Session 1\");
}" > src/main.rs

# Run tests
cargo test

# Check specific piece
cargo test url_generator
cargo test image_processor
cargo test memory_monitor
```

---

## **What You're Learning**

- **Piece 1**: Basic Rust structs, methods, string formatting
- **Piece 2**: Async/await, error handling with `?`, external crate APIs
- **Piece 3**: System metrics, mutable state management

---

## **When You're Done**

You should have:
1. ✅ All tests passing
2. ✅ Can generate URLs
3. ✅ Can process a single image (see the file created)
4. ✅ Can monitor memory usage

**Next**: Show me your code or ask questions. Then I'll give you pieces 4-7 (naive loop, metrics collector, batched processor, channels).

---

## **Stuck? Quick Hints**

**"Borrow checker issues"**: You probably need `.clone()` or `.to_string()`
**"Can't use `?` in function"**: Add `-> Result<T>` return type
**"Async function not working"**: Need `#[tokio::main]` or `#[tokio::test]`
**"Image save fails"**: Make sure output directory exists first

Good luck! 🚀
