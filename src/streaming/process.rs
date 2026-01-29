use anyhow::Result;
use futures::future::join_all;
use image::{load_from_memory, DynamicImage};
use tokio::{sync::mpsc, task::spawn_blocking, time::Instant};
use tracing::{debug, info};

use crate::streaming::download::ImageData;

pub struct ProcessedImage {
    pub url: String,
    pub image: DynamicImage,
    pub download_ms: u128,
    pub resize_ms: u128,
}

pub async fn process_stage(
    mut input: mpsc::Receiver<ImageData>,
    output: mpsc::Sender<ProcessedImage>,
) -> Result<()> {
    let mut handles = vec![];
    let mut processed = 0usize;
    info!("process stage started");
    while let Some(img_data) = input.recv().await {
        let start_resize = Instant::now();
        let local_sender = output.clone();
        processed += 1;
        debug!(url = %img_data.url, "processing image");

        let handle = spawn_blocking(move || {
            let original_img = load_from_memory(&img_data.bytes).unwrap();
            let resized_img =
                original_img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
            let resize_time = start_resize.elapsed().as_millis();

            let processed_img_data = ProcessedImage {
                url: img_data.url,
                image: resized_img,
                download_ms: img_data.download_ms,
                resize_ms: resize_time,
            };

            local_sender.blocking_send(processed_img_data).unwrap();
        });

        handles.push(handle);
    }

    join_all(handles).await;

    info!(processed, "process stage complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn processes_images() {
        let (input_tx, input_rx) = mpsc::channel(10);
        let (output_tx, mut output_rx) = mpsc::channel(10);

        tokio::spawn(async move {
            let bytes = reqwest::get("https://picsum.photos/seed/1/400/300")
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec();

            input_tx
                .send(ImageData {
                    url: "test".to_string(),
                    bytes,
                    download_ms: 0,
                })
                .await
                .unwrap();
        });

        tokio::spawn(async move {
            process_stage(input_rx, output_tx).await.unwrap();
        });

        if let Some(processed) = output_rx.recv().await {
            assert_eq!(processed.image.width(), 256);
            assert_eq!(processed.image.height(), 256);
        }
    }
}
