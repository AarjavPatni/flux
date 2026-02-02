# Flux

*This project was inspired by Daft's blog post: [Processing 300K Images Without OOM](https://www.daft.ai/blog/processing-300k-images-without-oom)*

Streaming image processing in Rust. Compare naive, batched, and streaming pipelines while tracking time and memory.

**Project writeup:** [Flux: Image Processing Without OOM](https://www.aarjav.xyz/blog/flux-image-processing)

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))

## Installation

```bash
git clone https://github.com/AarjavPatni/flux.git
cd flux
cargo build --release
```

## What It Does

- Downloads images from Lorem Picsum
- Processes images via three approaches: naive, batched, streaming
- Tracks time, memory, and throughput
- Outputs a comparison table

## Architecture

**Naive (src/naive + src/image_processor)**

```mermaid
flowchart LR
  A[URLs
url_generator.rs] --> B[process_single_image
image_processor.rs]
  B --> C[download]
  B --> D[decode]
  B --> E[resize]
  B --> F[save]
  F --> G[Output files
data/processed/naive]
```

**Batched (src/batched)**

```mermaid
flowchart LR
  A[URLs
url_generator.rs] --> B[Batches
batched/processor.rs]
  B --> C[Spawn tasks per batch]
  C --> D[process_single_image
image_processor.rs]
  D --> E[Output files
data/processed/batched]
  C --> F[Global memory sampler
memory_monitor.rs]
```

**Streaming (src/streaming)**

```mermaid
flowchart LR
  A[URLs
url_generator.rs] --> B[Download stage
streaming/download.rs]
  B -->|mpsc channel| C[Process stage
streaming/process.rs]
  C -->|mpsc channel| D[Save stage
streaming/pipeline.rs]
  D --> E[Output files
data/processed/streaming]
  B --> F[Semaphore
download concurrency]
  C --> G[Semaphore
process concurrency]
  D --> H[Global memory sampler
memory_monitor.rs]
```

## Usage

```bash
# Run all pipelines with specified image count (default: 200)
cargo run --release -- <image_count>

# Examples
RUST_LOG=info cargo run --release -- 1000
RUST_LOG=info cargo run --release -- 20
```

**Logging levels:** Set `RUST_LOG=info` for summaries or `RUST_LOG=debug` for per-image details.

**Outputs are written to:**

```
data/processed/naive
data/processed/batched
data/processed/streaming
```

## Example Results (1k images)

**Test machine**

- MacBook Pro (14-inch, 2023)
- Apple M2 Pro (10-core), 16 GB RAM
- macOS Sequoia 15.7.3 (arm64)

| Approach  | Images | Time (ms) | Peak Mem (MB) | Avg DL (ms) | Avg Resize (ms) | Throughput (img/s) |
| --------- | ------ | --------- | ------------- | ----------- | --------------- | ------------------ |
| naive     | 1000   | 850341    | 91            | 302         | 415             | 1.18               |
| batched   | 1000   | 146564    | 210           | 548         | 500             | 6.82               |
| streaming | 1000   | 77831     | 263           | 289         | 769             | 12.85              |

**Speedups (time):**

- Batched is **5.80x faster** than naive
- Streaming is **10.93x faster** than naive
- Streaming is **1.88x faster** than batched

**Throughput (img/s):**

- Batched is **5.80x higher** than naive
- Streaming is **10.93x higher** than naive
- Streaming is **1.88x higher** than batched

**Peak memory (MB):**

- Batched is **2.31x higher** than naive
- Streaming is **2.89x higher** than naive
- Streaming is **1.25x higher** than batched

### Implementation Notes

**Memory usage results:** Streaming shows higher peaks due to overlapping pipeline stages with data in-flight.

**Concurrency parameters (for 1k run):**

- Batched: batch size 10
- Streaming: download concurrency 20, process concurrency 4, channel capacity 50

## Project Structure

```
src/
├── main.rs                  # CLI + pipeline runner
├── url_generator.rs         # Lorem Picsum URLs
├── image_processor.rs       # Single-image baseline
├── memory_monitor.rs        # Process memory tracking
├── naive/                   # Sequential pipeline
├── batched/                 # Batched pipeline
└── streaming/               # Streaming pipeline (channels + backpressure)
```

## Notes

- Network variability affects timings
- Picsum images are random but stable via seeds
- Streaming pipeline uses bounded channels for backpressure

