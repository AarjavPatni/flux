# Session 2: Naive Loop & Batching (Estimated: 2 hours)

## Overview
Build sequential and batched processors. **Goal**: See the OOM problem, then fix it with batching.

---

## **Piece 4: Naive Sequential Processor** (30 min)

### What & Why
Process images one-by-one in a simple loop. This is the baseline that shows **why streaming matters**.

You'll see:
- Memory grows linearly with image count
- No parallelism (slow)
- Simple but doesn't scale

### Your Task
```rust
// src/naive/processor.rs

use crate::{image_processor, url_generator::UrlGenerator, memory_monitor::MemoryMonitor};
use anyhow::Result;
use std::path::Path;

pub struct ProcessingStats {
    pub total_images: usize,
    pub total_time_ms: u64,
    pub peak_memory_mb: u64,
    pub avg_download_ms: u64,
    pub avg_resize_ms: u64,
}

/// Naive sequential processor: process images one at a time
pub async fn process_naive(
    count: usize,
    output_dir: &Path,
) -> Result<ProcessingStats> {
    todo!("Implement naive sequential processing")
    
    // Steps:
    // 1. Create UrlGenerator and generate URLs
    // 2. Create MemoryMonitor
    // 3. Track start time
    // 4. Loop through URLs:
    //    - Process each image with image_processor::process_single_image
    //    - Track peak memory after each image
    //    - Collect metrics
    // 5. Calculate stats (total time, averages, peak memory)
    // 6. Return ProcessingStats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[tokio::test]
    async fn processes_images_sequentially() {
        let output = Path::new("test_output_naive");
        fs::create_dir_all(output).unwrap();
        
        let stats = process_naive(5, output).await.unwrap();
        
        assert_eq!(stats.total_images, 5);
        assert!(stats.total_time_ms > 0);
        assert!(stats.peak_memory_mb > 0);
        
        fs::remove_dir_all(output).unwrap();
    }
}
```

### Key Pattern: Tracking Peak Memory
```rust
let mut monitor = MemoryMonitor::new();
let mut peak_memory_mb = 0u64;

for url in urls {
    // Process image
    process_single_image(&url, output_dir).await?;
    
    // Track peak
    let current = monitor.current_usage_mb();
    peak_memory_mb = peak_memory_mb.max(current);
}
```

### Key Pattern: Calculating Averages
```rust
let metrics_vec: Vec<ImageMetrics> = vec![/* collected metrics */];

let total_download: u64 = metrics_vec.iter().map(|m| m.download_ms).sum();
let avg_download = total_download / metrics_vec.len() as u64;
```

### Acceptance
- Test passes
- Run with 10 images: prints stats showing ~5-10 seconds total
- Memory increases during processing

---

## **Piece 5: Metrics Collector** (20 min)

### What & Why
Save processing stats to CSV for later analysis. You'll compare naive vs batched vs streaming.

### Your Task
```rust
// src/metrics.rs

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ProcessingRun {
    pub approach: String,      // "naive", "batched", "streaming"
    pub image_count: usize,
    pub total_time_ms: u64,
    pub peak_memory_mb: u64,
    pub avg_download_ms: u64,
    pub avg_resize_ms: u64,
    pub throughput: f64,       // images per second
}

impl ProcessingRun {
    pub fn new(
        approach: &str,
        image_count: usize,
        total_time_ms: u64,
        peak_memory_mb: u64,
        avg_download_ms: u64,
        avg_resize_ms: u64,
    ) -> Self {
        let throughput = (image_count as f64 / total_time_ms as f64) * 1000.0;
        
        Self {
            approach: approach.to_string(),
            image_count,
            total_time_ms,
            peak_memory_mb,
            avg_download_ms,
            avg_resize_ms,
            throughput,
        }
    }
}

pub struct MetricsCollector {
    runs: Vec<ProcessingRun>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        todo!()
    }
    
    pub fn add_run(&mut self, run: ProcessingRun) {
        todo!()
    }
    
    /// Save all runs to CSV
    pub fn save_csv(&self, path: &Path) -> Result<()> {
        todo!()
        
        // Format:
        // approach,image_count,total_time_ms,peak_memory_mb,avg_download_ms,avg_resize_ms,throughput
        // naive,100,15234,450,230,290,6.56
    }
    
    /// Print comparison table to stdout
    pub fn print_comparison(&self) {
        todo!()
        
        // Print something like:
        // Approach     | Images | Time(ms) | Peak Mem(MB) | Throughput(img/s)
        // -------------|--------|----------|--------------|------------------
        // naive        | 100    | 15234    | 450          | 6.56
        // batched      | 100    | 8456     | 180          | 11.82
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn saves_csv() {
        let mut collector = MetricsCollector::new();
        
        collector.add_run(ProcessingRun::new(
            "naive", 100, 15000, 450, 230, 290
        ));
        collector.add_run(ProcessingRun::new(
            "batched", 100, 8000, 180, 220, 285
        ));
        
        let path = Path::new("test_metrics.csv");
        collector.save_csv(path).unwrap();
        
        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("naive"));
        assert!(contents.contains("batched"));
        
        fs::remove_file(path).unwrap();
    }
}
```

### Key Pattern: Writing CSV
```rust
use std::fs::File;
use std::io::Write;

let mut file = File::create(path)?;
writeln!(file, "header1,header2,header3")?;
writeln!(file, "{},{},{}", val1, val2, val3)?;
```

### Acceptance
- Test passes
- Creates valid CSV file
- Can open in spreadsheet and see data

---

## **Piece 6: Batched Processor** (40 min)

### What & Why
Process images in **batches** to control memory. This is the manual solution before full streaming.

**Key concept**: Process 10 images, wait for completion, process next 10. Memory doesn't explode.

### Your Task
```rust
// src/batched/processor.rs

use crate::{image_processor, url_generator::UrlGenerator, memory_monitor::MemoryMonitor};
use anyhow::Result;
use std::path::Path;

pub struct BatchedStats {
    pub total_images: usize,
    pub batch_size: usize,
    pub total_time_ms: u64,
    pub peak_memory_mb: u64,
    pub avg_download_ms: u64,
    pub avg_resize_ms: u64,
}

/// Batched processor: process images in chunks
pub async fn process_batched(
    count: usize,
    batch_size: usize,
    output_dir: &Path,
) -> Result<BatchedStats> {
    todo!("Implement batched processing")
    
    // Steps:
    // 1. Generate URLs
    // 2. Split into batches (use .chunks(batch_size))
    // 3. For each batch:
    //    - Spawn async tasks with tokio::spawn
    //    - Wait for all tasks in batch with futures::future::join_all
    //    - Track memory after each batch
    // 4. Calculate stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[tokio::test]
    async fn processes_in_batches() {
        let output = Path::new("test_output_batched");
        fs::create_dir_all(output).unwrap();
        
        let stats = process_batched(10, 3, output).await.unwrap();
        
        assert_eq!(stats.total_images, 10);
        assert_eq!(stats.batch_size, 3);
        assert!(stats.total_time_ms > 0);
        
        fs::remove_dir_all(output).unwrap();
    }
}
```

### Key Pattern: Processing Batches
```rust
use futures::future::join_all;

for batch in urls.chunks(batch_size) {
    // Spawn tasks for this batch
    let mut tasks = vec![];
    for url in batch {
        let url_clone = url.clone();
        let output_clone = output_dir.to_path_buf();
        
        let task = tokio::spawn(async move {
            image_processor::process_single_image(&url_clone, &output_clone).await
        });
        
        tasks.push(task);
    }
    
    // Wait for batch to complete
    let results = join_all(tasks).await;
    
    // Extract metrics from results
    for result in results {
        let metrics = result??; // First ? is JoinError, second is our Result
        // Collect metrics...
    }
}
```

### Acceptance
- Test passes
- Run with batch_size=5: should be faster than naive
- Peak memory lower than naive (processes in waves)

---

## **Piece 7: Channel Basics** (30 min)

### What & Why
Learn Tokio channels before building the streaming pipeline. Channels are how stages communicate.

### Your Task
```rust
// src/streaming/channel_demo.rs

use tokio::sync::mpsc;
use anyhow::Result;

/// Demonstrate basic channel usage: producer sends, consumer receives
pub async fn channel_demo() -> Result<()> {
    todo!("Implement channel demo")
    
    // Steps:
    // 1. Create channel with mpsc::channel(10) - capacity of 10
    // 2. Spawn producer task that sends numbers 0..20
    // 3. Spawn consumer task that receives and prints
    // 4. Wait for both to complete
}

/// Demonstrate backpressure: bounded channel blocks when full
pub async fn backpressure_demo() -> Result<()> {
    todo!("Implement backpressure demo")
    
    // Steps:
    // 1. Create channel with capacity 3
    // 2. Producer sends fast (no delay)
    // 3. Consumer receives slow (sleep 100ms between receives)
    // 4. Observe: producer blocks when channel is full
    // 
    // Print messages like:
    // "Producer: sending 0"
    // "Producer: sending 1"
    // "Producer: sending 2"
    // "Producer: sending 3" <- blocks here until consumer receives
    // "Consumer: received 0"
    // "Producer: sending 4"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn basic_channel_works() {
        channel_demo().await.unwrap();
    }
    
    #[tokio::test]
    async fn backpressure_works() {
        backpressure_demo().await.unwrap();
    }
}
```

### Key Pattern: Producer-Consumer
```rust
use tokio::sync::mpsc;

// Create channel
let (tx, mut rx) = mpsc::channel(10); // capacity = 10

// Producer
let producer = tokio::spawn(async move {
    for i in 0..5 {
        tx.send(i).await.unwrap();
        println!("Sent: {}", i);
    }
});

// Consumer
let consumer = tokio::spawn(async move {
    while let Some(val) = rx.recv().await {
        println!("Received: {}", val);
    }
});

// Wait for both
producer.await.unwrap();
consumer.await.unwrap();
```

### Key Pattern: Backpressure
```rust
// Small capacity = backpressure
let (tx, mut rx) = mpsc::channel(3);

// Fast producer
tokio::spawn(async move {
    for i in 0..10 {
        println!("Trying to send {}", i);
        tx.send(i).await.unwrap(); // Blocks when channel full!
        println!("Sent {}", i);
    }
});

// Slow consumer
tokio::spawn(async move {
    while let Some(val) = rx.recv().await {
        tokio::time::sleep(Duration::from_millis(100)).await;
        println!("Received {}", val);
    }
});
```

### Acceptance
- Both tests pass
- `backpressure_demo()` shows producer blocking when consumer is slow
- You understand how bounded channels prevent memory explosion

---

## Module Setup

Create the files:
```bash
mkdir -p src/naive src/batched src/streaming
touch src/naive/processor.rs
touch src/batched/processor.rs
touch src/streaming/channel_demo.rs
touch src/metrics.rs

# Add to main.rs:
# mod naive;
# mod batched;
# mod streaming;
# mod metrics;
```

Update `src/main.rs`:
```rust
mod url_generator;
mod image_processor;
mod memory_monitor;
mod naive;
mod batched;
mod streaming;
mod metrics;

fn main() {
    println!("Image Processor - Session 2");
}
```

Add module declarations:
```rust
// src/naive/mod.rs
pub mod processor;

// src/batched/mod.rs
pub mod processor;

// src/streaming/mod.rs
pub mod channel_demo;
```

---

## What You're Learning

- **Piece 4**: Sequential async processing, memory tracking
- **Piece 5**: File I/O, CSV formatting, data analysis
- **Piece 6**: Concurrent batch processing, `tokio::spawn`, `join_all`
- **Piece 7**: **Critical foundation** - channels, backpressure, producer-consumer pattern

---

## When You're Done

You should be able to:
1. ✅ Process 10 images with naive approach (slow, high memory)
2. ✅ Process 10 images with batched approach (faster, lower memory)
3. ✅ Save comparison to CSV
4. ✅ Understand how channels prevent OOM

**Next**: Show me your code. Then I'll give you pieces 8-10 (full streaming pipeline with download/process/save stages).

Good luck! 🚀
