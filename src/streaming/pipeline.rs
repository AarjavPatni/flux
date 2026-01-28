use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::{spawn, sync::mpsc, time::Instant, try_join};

use crate::{
    streaming::{
        download::{download_stage, ImageData},
        process::{process_stage, ProcessedImage},
    },
    url_generator::UrlGenerator,
};

pub struct StreamingStats {
    pub total_images: usize,
    pub total_time_ms: u64,
    // pub peak_memory_mb: u64,
    pub avg_download_ms: u64,
    pub avg_resize_ms: u64,
}

async fn save_stage(
    mut input: mpsc::Receiver<ProcessedImage>,
    output_dir: &Path,
) -> Result<(u64, u64)> {
    // TODO: What if there's a situation where there's no more data and the channel closes, this function returns, but then the data gets added later? Is this kind of situation possible?
    let mut total_download_ms = 0;
    let mut total_resize_ms = 0;
    let mut image_count: u128 = 0;

    while let Some(image_data) = input.recv().await {
        let filename = format!("{:x}.jpg", Sha256::digest(image_data.url.as_bytes()));
        image_data.image.save(output_dir.join(filename))?;
        total_download_ms += image_data.download_ms;
        total_resize_ms += image_data.resize_ms;
        image_count += 1;
    }

    anyhow::ensure!(image_count > 0, "no images processed");

    let avg_download_ms: u64 = (total_download_ms / image_count) as u64;
    let avg_resize_ms: u64 = (total_resize_ms / image_count) as u64;

    Ok((avg_download_ms, avg_resize_ms))
}

pub async fn process_streaming(
    count: usize,
    output_dir: &Path,
    download_concurrency: usize,
    channel_capacity: usize,
) -> Result<StreamingStats> {
    let start_time = Instant::now();
    let urls = UrlGenerator::new(count).generate();
    let output_pathbuf = output_dir.to_path_buf();

    let (download_tx, download_rx) = mpsc::channel::<ImageData>(channel_capacity);
    let (process_tx, process_rx) = mpsc::channel::<ProcessedImage>(channel_capacity);

    let download_task =
        spawn(async move { download_stage(urls, download_tx, download_concurrency).await });
    let process_task = spawn(async move { process_stage(download_rx, process_tx).await });
    let save_task = spawn(async move { save_stage(process_rx, &output_pathbuf).await });

    let (_, _, save_res) = try_join!(download_task, process_task, save_task)?;
    let (avg_download_ms, avg_resize_ms) = save_res?;

    let total_time_ms = start_time.elapsed().as_millis() as u64;

    // TODO: Add peak memory. Convert tracking in naive and batch to true peak as well.
    Ok(StreamingStats {
        total_images: count,
        total_time_ms,
        // peak_memory_mb: u64,
        avg_download_ms,
        avg_resize_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn streams_images() {
        let output = Path::new("test_output_streaming");
        fs::create_dir_all(output).unwrap();

        let stats = process_streaming(10, output, 3, 5).await.unwrap();

        assert_eq!(stats.total_images, 10);
        assert!(stats.total_time_ms > 0);

        fs::remove_dir_all(output).unwrap();
    }
}
