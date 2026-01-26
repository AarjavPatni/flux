// src/image_processor.rs

use anyhow::Result;
use std::{path::Path, time::Instant};
use image::{self, EncodableLayout};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct ImageMetrics {
    pub url: String,
    pub download_ms: u64,
    pub decode_ms: u64,
    pub resize_ms: u64,
    pub save_ms: u64,
    pub bytes_downloaded: usize,
}

/// Process a single image: download → decode → resize → save
pub async fn process_single_image(
    url: &str,
    output_dir: &Path,
) -> Result<ImageMetrics> {
    let request_start = Instant::now();
    let img_bytes = reqwest::get(url).await?.bytes().await?;
    let request_end = Instant::now();
    let request_metric = (request_end - request_start).as_millis() as u64;
    
    let decode_start = Instant::now();
    let img = image::load_from_memory(&img_bytes)?;
    let decode_end = Instant::now();
    let decode_metric = (decode_end - decode_start).as_millis() as u64;
    
    let resize_start = Instant::now();
    let resized_img = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
    let resize_end = Instant::now();
    let resize_metric = (resize_end - resize_start).as_millis() as u64;
    
    let filename = format!("{:x}.jpg", Sha256::digest(url.as_bytes()));
    
    let save_start = Instant::now();
    resized_img.save(output_dir.join(filename))?;
    let save_end = Instant::now();
    let save_metric = (save_end - save_start).as_millis() as u64;
    
    Ok(ImageMetrics {
        url: url.to_string(),
        download_ms: request_metric,
        decode_ms: decode_metric,
        resize_ms: resize_metric,
        save_ms: save_metric,
        bytes_downloaded: img_bytes.len(),
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
