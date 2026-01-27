# Session 3: Streaming Pipeline (Estimated: 2-3 hours)

## Overview
Build a 3-stage streaming pipeline with automatic backpressure:
- **Stage 1**: Download images (async I/O)
- **Stage 2**: Resize images (CPU-bound, use `spawn_blocking`)
- **Stage 3**: Save to disk (async I/O)

All stages run concurrently, connected by bounded channels.

---

## **Piece 8: Download Stage** (30 min)

### Goal
Async stage that downloads images with controlled concurrency using a semaphore.

### Your Task
```rust
// src/streaming/download.rs

use tokio::sync::{mpsc, Semaphore};
use std::sync::Arc;
use anyhow::Result;

pub struct ImageData {
    pub url: String,
    pub bytes: Vec<u8>,
}

pub async fn download_stage(
    urls: Vec<String>,
    output: mpsc::Sender<ImageData>,
    concurrency: usize,
) -> Result<()> {
    todo!()
    
    // Pattern:
    // 1. Create semaphore with capacity = concurrency
    // 2. For each URL, spawn task that:
    //    - Acquires semaphore permit
    //    - Downloads image with reqwest
    //    - Sends ImageData through channel
    //    - Drops permit (automatic)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn downloads_images() {
        let urls = vec![
            "https://picsum.photos/seed/1/400/300".to_string(),
            "https://picsum.photos/seed/2/400/300".to_string(),
        ];
        
        let (tx, mut rx) = mpsc::channel(10);
        
        tokio::spawn(async move {
            download_stage(urls, tx, 2).await.unwrap();
        });
        
        let mut count = 0;
        while let Some(data) = rx.recv().await {
            assert!(data.bytes.len() > 0);
            count += 1;
        }
        
        assert_eq!(count, 2);
    }
}
```

---

## **Piece 9: Process Stage** (30 min)

### Goal
CPU-bound stage that resizes images using `spawn_blocking`.

### Your Task
```rust
// src/streaming/process.rs

use tokio::sync::mpsc;
use tokio::task;
use anyhow::Result;
use image::DynamicImage;

pub struct ProcessedImage {
    pub url: String,
    pub image: DynamicImage,
}

pub async fn process_stage(
    mut input: mpsc::Receiver<crate::streaming::download::ImageData>,
    output: mpsc::Sender<ProcessedImage>,
) -> Result<()> {
    todo!()
    
    // Pattern:
    // 1. While receiving from input channel:
    // 2. Use tokio::task::spawn_blocking for CPU work:
    //    - Decode image from bytes
    //    - Resize to 256x256
    // 3. Send ProcessedImage through output channel
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::download::ImageData;
    
    #[tokio::test]
    async fn processes_images() {
        let (input_tx, input_rx) = mpsc::channel(10);
        let (output_tx, mut output_rx) = mpsc::channel(10);
        
        // Send test data
        tokio::spawn(async move {
            let bytes = reqwest::get("https://picsum.photos/seed/1/400/300")
                .await.unwrap()
                .bytes().await.unwrap()
                .to_vec();
            
            input_tx.send(ImageData {
                url: "test".to_string(),
                bytes,
            }).await.unwrap();
        });
        
        tokio::spawn(async move {
            process_stage(input_rx, output_tx).await.unwrap();
        });
        
        if let Some(processed) = output_rx.recv().await {
            assert_eq!(processed.image.width(), 256);
            assert_eq!(processed.image.height(), 256);
        }
    }
}
```

---

## **Piece 10: Full Pipeline** (1 hour)

### Goal
Orchestrate all 3 stages with proper channels and metrics tracking.

### Your Task
```rust
// src/streaming/pipeline.rs

use tokio::sync::mpsc;
use anyhow::Result;
use std::path::Path;

pub struct StreamingStats {
    pub total_images: usize,
    pub total_time_ms: u64,
    pub peak_memory_mb: u64,
    pub avg_download_ms: u64,
    pub avg_resize_ms: u64,
}

pub async fn process_streaming(
    count: usize,
    output_dir: &Path,
    download_concurrency: usize,
    channel_capacity: usize,
) -> Result<StreamingStats> {
    todo!()
    
    // Architecture:
    // 1. Generate URLs
    // 2. Create channels:
    //    - (download_tx, download_rx) for download -> process
    //    - (process_tx, process_rx) for process -> save
    // 3. Spawn 3 stages:
    //    - download_stage(urls, download_tx, concurrency)
    //    - process_stage(download_rx, process_tx)
    //    - save_stage(process_rx, output_dir)
    // 4. Wait for all to complete
    // 5. Calculate and return stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[tokio::test]
    async fn streams_images() {
        let output = Path::new("test_output_streaming");
        fs::create_dir_all(output).unwrap();
        
        let stats = process_streaming(10, output, 3, 5).await.unwrap();
        
        assert_eq!(stats.total_images, 10);
        assert!(stats.total_time_ms > 0);
        
        fs::remove_dir_all(output).unwrap();
    }
}
```

---

## Module Setup

```bash
touch src/streaming/download.rs
touch src/streaming/process.rs
touch src/streaming/pipeline.rs

# Update src/streaming/mod.rs
```

---

## Key Concepts You'll Learn

**Piece 8**: Semaphore for concurrency control, async I/O
**Piece 9**: `spawn_blocking` for CPU work, channel forwarding
**Piece 10**: Multi-stage pipeline, backpressure in action

---

## When You're Done

You'll have a full streaming pipeline that:
- Downloads images concurrently (controlled by semaphore)
- Processes them with `spawn_blocking` (CPU work)
- Saves them to disk
- All stages run simultaneously with automatic backpressure

**Next**: Compare naive vs batched vs streaming performance!

Go! 🚀
