# Flux

Learn streaming data processing by building a concurrent image processor in Rust.

## Goal
Understand how to process 100K+ images without running out of memory by implementing:
1. Naive sequential processor (see the problem)
2. Batched processor (manual control)
3. Streaming pipeline (automatic backpressure)
4. Live TUI monitoring

## Quick Start

```bash
# Session 1: Implement pieces 1-3
cargo test

# Run individual tests
cargo test url_generator
cargo test image_processor
cargo test memory_monitor
```

See `IMPLEMENTATION_GUIDE.md` for detailed instructions.

## Project Structure

```
src/
├── main.rs              # Entry point
├── url_generator.rs     # Generate Lorem Picsum URLs
├── image_processor.rs   # Single image processing
├── memory_monitor.rs    # Track memory usage
├── naive/               # (coming in session 2)
├── batched/             # (coming in session 2)
└── streaming/           # (coming in session 3)
```

## Learning Path

**Session 1** (1 hour): Basic pieces
- URL generation
- Single image processing  
- Memory monitoring

**Session 2** (2 hours): Sequential & batched
- Naive loop (see OOM problem)
- Metrics collection
- Batched processing
- Channel fundamentals

**Session 3** (2-3 hours): Streaming pipeline
- Async download stage
- Blocking process stage
- Full pipeline integration
- TUI monitoring

**Session 4**: Polish & benchmark
- Compare all approaches
- Generate performance charts
- Document findings
