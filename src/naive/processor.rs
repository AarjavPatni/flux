use crate::{
    image_processor::process_single_image,
    url_generator::UrlGenerator,
};
use anyhow::Result;
use std::{cmp::max, path::Path};
use tokio::time::Instant;
use tracing::info;

pub struct ProcessingStats {
    pub total_images: usize,
    pub total_time_ms: u64,
    pub peak_memory_mb: u64,
    pub avg_download_ms: u64,
    pub avg_resize_ms: u64,
}

pub async fn process_naive(count: usize, output_dir: &Path) -> Result<ProcessingStats> {
    info!(count, "starting naive processing");

    let url_generator = UrlGenerator::new(count);
    let urls = url_generator.generate();
    let mut total_download_time: u64 = 0;
    let mut total_resize_time: u64 = 0;
    let total_time: u64;
    let mut peak_memory_usage: u64 = 0;

    let start_time = Instant::now();
    for (index, u) in urls.iter().enumerate() {
        info!(index = index + 1, total = count, url = %u, "processing image");

        let metric = process_single_image(&u, output_dir).await.unwrap();
        peak_memory_usage = max(metric.peak_memory_mb, peak_memory_usage);
        total_download_time += metric.download_ms;
        total_resize_time += metric.resize_ms;

        info!(
            download_ms = metric.download_ms,
            resize_ms = metric.resize_ms,
            memory_mb = metric.peak_memory_mb,
            "image processed"
        );
    }
    let end_time = Instant::now();

    total_time = (end_time - start_time).as_millis() as u64;

    info!(
        total_time_ms = total_time,
        peak_memory_mb = peak_memory_usage,
        avg_download_ms = total_download_time / count as u64,
        avg_resize_ms = total_resize_time / count as u64,
        "naive processing complete"
    );

    Ok(ProcessingStats {
        total_images: count,
        total_time_ms: total_time,
        peak_memory_mb: peak_memory_usage,
        avg_download_ms: total_download_time / count as u64,
        avg_resize_ms: total_resize_time / count as u64,
    })
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
