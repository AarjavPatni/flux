use anyhow::Result;
use futures::future::join_all;
use std::sync::Arc;
use tokio::{
    spawn,
    sync::{mpsc, Semaphore},
};

pub struct ImageData {
    pub url: String,
    pub bytes: Vec<u8>,
}

pub async fn download_stage(
    urls: Vec<String>,
    output: mpsc::Sender<ImageData>,
    concurrency: usize,
) -> Result<()> {
    let sem = Arc::new(Semaphore::new(concurrency));
    let mut handles = vec![];

    for u in urls {
        let sem_clone = Arc::clone(&sem);
        let output_clone = output.clone();

        let handle = spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            let img_bytes = reqwest::get(&u)
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec();

            output_clone.send(ImageData {
                url: u,
                bytes: img_bytes,
            }).await.unwrap();
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
    async fn downloads_images() {
        let urls = vec![
            "https://picsum.photos/seed/1/400/300".to_string(),
            "https://picsum.photos/seed/2/400/300".to_string(),
        ];

        let (tx, mut rx) = mpsc::channel(10);

        tokio::spawn(async move {
            download_stage(urls, tx, 2).await.unwrap();
        });

        let mut count = 0;
        while let Some(data) = rx.recv().await {
            assert!(data.bytes.len() > 0);
            count += 1;
        }

        assert_eq!(count, 2);
    }
}
