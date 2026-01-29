mod url_generator;
mod image_processor;
mod memory_monitor;
mod naive;
mod batched;
mod streaming;
mod metrics;

use std::{env, fs, path::Path};

use anyhow::Result;
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};

use crate::{
    batched::processor::process_batched,
    metrics::{MetricsCollector, ProcessingRun},
    naive::processor::process_naive,
    streaming::pipeline::process_streaming,
};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();

    let count = parse_count_arg().unwrap_or(200);

    info!(count, "flux image processor started");

    let base_dir = Path::new("data/processed");
    fs::create_dir_all(base_dir)?;

    let naive_dir = base_dir.join("naive");
    let batched_dir = base_dir.join("batched");
    let streaming_dir = base_dir.join("streaming");
    fs::create_dir_all(&naive_dir)?;
    fs::create_dir_all(&batched_dir)?;
    fs::create_dir_all(&streaming_dir)?;

    if tracing::enabled!(tracing::Level::INFO) {
        println!();
    }
    let naive_stats = process_naive(count, &naive_dir).await?;
    info!(
        total_time_ms = naive_stats.total_time_ms,
        peak_memory_mb = naive_stats.peak_memory_mb,
        avg_download_ms = naive_stats.avg_download_ms,
        avg_resize_ms = naive_stats.avg_resize_ms,
        "naive summary"
    );

    if tracing::enabled!(tracing::Level::INFO) {
        println!();
    }
    let batched_stats = process_batched(count, 10, &batched_dir).await?;
    info!(
        total_time_ms = batched_stats.total_time_ms,
        peak_memory_mb = batched_stats.peak_memory_mb,
        avg_download_ms = batched_stats.avg_download_ms,
        avg_resize_ms = batched_stats.avg_resize_ms,
        "batched summary"
    );

    if tracing::enabled!(tracing::Level::INFO) {
        println!();
    }
    let streaming_stats = process_streaming(count, &streaming_dir, 8, 10, 10).await?;
    info!(
        total_time_ms = streaming_stats.total_time_ms,
        peak_memory_mb = streaming_stats.peak_memory_mb,
        avg_download_ms = streaming_stats.avg_download_ms,
        avg_resize_ms = streaming_stats.avg_resize_ms,
        "streaming summary"
    );

    let mut collector = MetricsCollector::new();
    collector.add_run(ProcessingRun::new(
        "naive",
        naive_stats.total_images,
        naive_stats.total_time_ms,
        naive_stats.peak_memory_mb,
        naive_stats.avg_download_ms,
        naive_stats.avg_resize_ms,
    ));
    collector.add_run(ProcessingRun::new(
        "batched",
        batched_stats.total_images,
        batched_stats.total_time_ms,
        batched_stats.peak_memory_mb,
        batched_stats.avg_download_ms,
        batched_stats.avg_resize_ms,
    ));
    collector.add_run(ProcessingRun::new(
        "streaming",
        streaming_stats.total_images,
        streaming_stats.total_time_ms,
        streaming_stats.peak_memory_mb,
        streaming_stats.avg_download_ms,
        streaming_stats.avg_resize_ms,
    ));

    collector.print_comparison();

    Ok(())
}

fn parse_count_arg() -> Option<usize> {
    let mut args = env::args().skip(1);
    let count = args.next()?;
    match count.parse::<usize>() {
        Ok(value) => Some(value),
        Err(_) => {
            warn!(arg = %count, "invalid count arg, falling back to default");
            None
        }
    }
}
