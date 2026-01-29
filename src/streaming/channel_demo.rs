// src/streaming/channel_demo.rs

use std::time::Duration;

use anyhow::Result;
use tokio::{sync::mpsc, time::sleep};
use tracing::info;

pub async fn channel_demo() -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<u64>(20);

    let producer = tokio::spawn(async move {
        for i in 1..=20 {
            tx.send(i).await.unwrap();
            info!(value = i, "producer sent");
        }
    });

    let consumer = tokio::spawn(async move {
        while let Some(val) = rx.recv().await {
            info!(value = val, "consumer received");
        }
    });

    producer.await?;
    consumer.await?;

    Ok(())
}

pub async fn backpressure_demo() -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<u64>(3);

    let producer = tokio::spawn(async move {
        for i in 1..=20 {
            tx.send(i).await.unwrap();
            info!(value = i, "producer sent");
        }
    });

    let consumer = tokio::spawn(async move {
        while let Some(val) = rx.recv().await {
            info!(value = val, "consumer received");
            sleep(Duration::from_millis(100)).await;
        }
    });

    producer.await?;
    consumer.await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic_channel_works() {
        channel_demo().await.unwrap();
    }

    #[tokio::test]
    async fn backpressure_works() {
        backpressure_demo().await.unwrap();
    }
}
