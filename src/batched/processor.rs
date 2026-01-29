use crate::{
    image_processor::process_single_image, memory_monitor::MemoryMonitor,
    url_generator::UrlGenerator,
};
use anyhow::Result;
use futures::future::join_all;
use std::{
    cmp::max,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    spawn,
    sync::Mutex,
    time::{self, sleep},
};

pub struct BatchedStats {
    pub total_images: usize,
    pub batch_size: usize,
    pub total_time_ms: u64,
    pub peak_memory_mb: u64,
    pub avg_download_ms: u64,
    pub avg_resize_ms: u64,
}

pub async fn process_batched(
    count: usize,
    batch_size: usize,
    output_dir: &Path,
) -> Result<BatchedStats> {
    println!("Starting batch processing of {} images", count);

    let url_generator = UrlGenerator::new(count);
    let urls = url_generator.generate();
    let (mut total_download_time, mut total_resize_time, mut total_time_ms) = (0, 0, 0);

    let peak_memory_mb = Arc::new(AtomicU64::new(0));
    let peak_clone = Arc::clone(&peak_memory_mb);

    let monitor_handle = spawn(async move {
        let mut memory_monitor = MemoryMonitor::new();
        loop {
            let curr_usage = memory_monitor.current_usage_mb();
            peak_clone.store(
                max(curr_usage, peak_clone.load(Ordering::Relaxed)),
                Ordering::Relaxed,
            );
            sleep(Duration::from_millis(100)).await;
        }
    });

    for batch in urls.chunks(batch_size) {
        let mut batch_tasks = vec![];
        let start_time = time::Instant::now();

        for url in batch {
            let owned_url = url.clone();
            let owned_path = output_dir.to_path_buf();

            batch_tasks.push(spawn(async move {
                let task_metric = process_single_image(&owned_url, &owned_path, None)
                    .await
                    .unwrap();

                (task_metric.download_ms, task_metric.resize_ms)
            }));
        }

        let batch_results = join_all(batch_tasks).await;
        let batch_duration = start_time.elapsed().as_millis() as u64;
        total_time_ms += batch_duration;

        for res in batch_results {
            let (task_download, task_resize) = res?;
            total_download_time += task_download;
            total_resize_time += task_resize;
        }
    }

    monitor_handle.abort();
    let peak_memory_mb = peak_memory_mb.load(Ordering::Relaxed);

    println!("\nBatch processing complete:");
    println!("  Total time: {}ms", total_time_ms);
    println!("  Peak memory: {}MB", peak_memory_mb);
    println!("  Avg download: {}ms", total_download_time / count as u64);
    println!("  Avg resize: {}ms", total_resize_time / count as u64);

    Ok(BatchedStats {
        total_images: count,
        batch_size,
        total_time_ms,
        peak_memory_mb,
        avg_download_ms: total_download_time / count as u64,
        avg_resize_ms: total_resize_time / count as u64,
    })
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
