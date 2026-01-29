// src/image_processor.rs

use anyhow::Result;
use image;
use sha2::{Digest, Sha256};
use std::{
    cmp::max,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{spawn, time::sleep};

use crate::memory_monitor::MemoryMonitor;

#[derive(Debug, Clone)]
pub struct ImageMetrics {
    pub url: String,
    pub download_ms: u64,
    pub decode_ms: u64,
    pub resize_ms: u64,
    pub save_ms: u64,
    pub bytes_downloaded: usize,
    pub peak_memory_mb: u64,
}

/// Process a single image: download → decode → resize → save
pub async fn process_single_image(url: &str, output_dir: &Path) -> Result<ImageMetrics> {
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

    let download_start = Instant::now();
    let img_bytes = reqwest::get(url).await?.bytes().await?;
    let download_end = Instant::now();
    let download_ms = (download_end - download_start).as_millis() as u64;

    let decode_start = Instant::now();
    let img = image::load_from_memory(&img_bytes)?;
    let decode_end = Instant::now();
    let decode_ms = (decode_end - decode_start).as_millis() as u64;

    let resize_start = Instant::now();
    let resized_img = img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
    let resize_end = Instant::now();
    let resize_ms = (resize_end - resize_start).as_millis() as u64;

    let filename = format!("{:x}.jpg", Sha256::digest(url.as_bytes()));

    let save_start = Instant::now();
    resized_img.save(output_dir.join(filename))?;
    let save_end = Instant::now();
    let save_ms = (save_end - save_start).as_millis() as u64;

    monitor_handle.abort();
    let peak_memory_mb = peak_memory_mb.load(Ordering::Relaxed);

    Ok(ImageMetrics {
        url: url.to_string(),
        download_ms,
        decode_ms,
        resize_ms,
        save_ms,
        bytes_downloaded: img_bytes.len(),
        peak_memory_mb,
    })
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

        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.download_ms > 0);
        assert!(metrics.bytes_downloaded > 0);

        // Cleanup
        fs::remove_dir_all(output).unwrap();
    }
}
