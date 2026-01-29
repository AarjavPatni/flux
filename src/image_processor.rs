// src/image_processor.rs

use anyhow::Result;
use image;
use sha2::{Digest, Sha256};
use std::{cmp::max, path::Path, time::Instant};

use crate::memory_monitor::{self, MemoryMonitor};

#[derive(Debug, Clone)]
pub struct ImageMetrics {
    pub url: String,
    pub download_ms: u64,
    pub decode_ms: u64,
    pub resize_ms: u64,
    pub save_ms: u64,
    pub bytes_downloaded: usize,
    pub peak_memory_mb: Option<u64>,
}

/// Process a single image: download → decode → resize → save
pub async fn process_single_image(
    url: &str,
    output_dir: &Path,
    mut memory_monitor: Option<&mut MemoryMonitor>,
) -> Result<ImageMetrics> {
    let mut peak_memory_mb: Option<u64> = None;

    let mut update_peak = || {
        if let Some(m) = memory_monitor.as_deref_mut() {
            let current = m.current_usage_mb();
            peak_memory_mb = Some(peak_memory_mb.map_or(current, |p| p.max(current)));
        }
    };

    let download_start = Instant::now();
    let img_bytes = reqwest::get(url).await?.bytes().await?;
    let download_end = Instant::now();
    update_peak();
    let download_ms = (download_end - download_start).as_millis() as u64;

    let decode_start = Instant::now();
    let img = image::load_from_memory(&img_bytes)?;
    let decode_end = Instant::now();
    update_peak();
    let decode_ms = (decode_end - decode_start).as_millis() as u64;

    let resize_start = Instant::now();
    let resized_img = img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
    let resize_end = Instant::now();
    update_peak();
    let resize_ms = (resize_end - resize_start).as_millis() as u64;

    let filename = format!("{:x}.jpg", Sha256::digest(url.as_bytes()));

    let save_start = Instant::now();
    resized_img.save(output_dir.join(filename))?;
    let save_end = Instant::now();
    update_peak();
    let save_ms = (save_end - save_start).as_millis() as u64;

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
        let mut memory_monitor = MemoryMonitor::new();
        let result = process_single_image(url, output, Some(&mut memory_monitor)).await;

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
