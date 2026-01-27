use crate::{
    image_processor::{self, process_single_image},
    memory_monitor::MemoryMonitor,
    url_generator::UrlGenerator,
};
use anyhow::Result;
use futures::future::join_all;
use std::{io::Error, path::Path, sync::Arc};
use tokio::{
    spawn,
    sync::Mutex,
    task::JoinHandle,
    time::{self, Instant},
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
    let (mut total_download_time, mut total_resize_time, mut total_time_ms, mut peak_memory_mb) =
        (0, 0, 0, 0);
    let memory_monitor = Arc::new(Mutex::new(MemoryMonitor::new()));

    for batch in urls.chunks(batch_size) {
        let mut batch_tasks = vec![];
        let start_time = time::Instant::now();

        for url in batch {
            let owned_url = url.clone();
            let owned_path = output_dir.to_path_buf();
            let monitor_clone = Arc::clone(&memory_monitor);

            batch_tasks.push(spawn(async move {
                let task_metric = process_single_image(&owned_url, &owned_path).await.unwrap();
                let task_memory_usage = monitor_clone.lock().await.current_usage_mb();

                (
                    task_metric.download_ms,
                    task_metric.resize_ms,
                    task_memory_usage,
                )
            }));
        }

        let batch_results = join_all(batch_tasks).await;
        let batch_duration = start_time.elapsed().as_millis() as u64;
        total_time_ms += batch_duration;

        for res in batch_results {
            let (task_download, task_resize, task_memory) = res?;
            total_download_time += task_download;
            total_resize_time += task_resize;

            if task_memory > peak_memory_mb {
                peak_memory_mb = task_memory;
            }
        }
    }

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
