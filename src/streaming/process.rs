use anyhow::Result;
use futures::future::join_all;
use image::{load_from_memory, DynamicImage};
use tokio::{sync::mpsc, task::spawn_blocking};

use crate::streaming::download::ImageData;

pub struct ProcessedImage {
    pub url: String,
    pub image: DynamicImage,
}

pub async fn process_stage(
    mut input: mpsc::Receiver<ImageData>,
    output: mpsc::Sender<ProcessedImage>,
) -> Result<()> {
    let mut handles = vec![];
    while let Some(img_data) = input.recv().await {
        let local_sender = output.clone();

        let handle = spawn_blocking(move || {
            let original_img = load_from_memory(&img_data.bytes).unwrap();
            let resized_img =
                original_img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);

            let processed_img_data = ProcessedImage {
                url: img_data.url,
                image: resized_img,
            };

            local_sender.blocking_send(processed_img_data).unwrap();
        });

        handles.push(handle);
    }

    join_all(handles).await;

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
