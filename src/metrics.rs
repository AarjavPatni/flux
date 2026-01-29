use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tabled::{settings::Style, Table, Tabled};

#[derive(Debug, Clone, Tabled)]
pub struct ProcessingRun {
    #[tabled(rename = "Approach")]
    pub approach: String,
    #[tabled(rename = "Images")]
    pub image_count: usize,
    #[tabled(rename = "Time (ms)")]
    pub total_time_ms: u64,
    #[tabled(rename = "Peak Mem (MB)")]
    pub peak_memory_mb: u64,
    #[tabled(rename = "Avg DL (ms)")]
    pub avg_download_ms: u64,
    #[tabled(rename = "Avg Resize (ms)")]
    pub avg_resize_ms: u64,
    #[tabled(rename = "Throughput (img/s)", display("display_throughput"))]
    pub throughput: f64,
}

fn display_throughput(throughput: &f64) -> String {
    format!("{:.2}", throughput)
}

impl ProcessingRun {
    pub fn new(
        approach: &str,
        image_count: usize,
        total_time_ms: u64,
        peak_memory_mb: u64,
        avg_download_ms: u64,
        avg_resize_ms: u64,
    ) -> Self {
        let throughput = (image_count as f64 / total_time_ms as f64) * 1000.0;

        Self {
            approach: approach.to_string(),
            image_count,
            total_time_ms,
            peak_memory_mb,
            avg_download_ms,
            avg_resize_ms,
            throughput,
        }
    }
}

pub struct MetricsCollector {
    runs: Vec<ProcessingRun>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        MetricsCollector { runs: vec![] }
    }

    pub fn add_run(&mut self, run: ProcessingRun) {
        self.runs.push(run);
    }

    pub fn save_csv(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "approach,image_count,total_time_ms,peak_memory_mb,avg_download_ms,avg_resize_ms,throughput")?;

        for run in &self.runs {
            writeln!(
                file,
                "{},{},{},{},{},{},{:.2}",
                run.approach,
                run.image_count,
                run.total_time_ms,
                run.peak_memory_mb,
                run.avg_download_ms,
                run.avg_resize_ms,
                run.throughput
            )?;
        }
        Ok(())
    }

    pub fn print_comparison(&self) {
        if self.runs.is_empty() {
            println!("No runs to compare");
            return;
        }

        println!("\nFlux Image Processor - Comparison\n");
        println!("{}\n", Table::new(&self.runs).with(Style::rounded()));

        let naive = self.runs.iter().find(|run| run.approach == "naive");
        let batched = self.runs.iter().find(|run| run.approach == "batched");
        let streaming = self.runs.iter().find(|run| run.approach == "streaming");

        if let (Some(naive), Some(batched)) = (naive, batched) {
            let speedup = naive.total_time_ms as f64 / batched.total_time_ms as f64;
            println!("Batched is {:.2}x faster than naive.", speedup);
        }

        if let (Some(naive), Some(streaming)) = (naive, streaming) {
            let speedup = naive.total_time_ms as f64 / streaming.total_time_ms as f64;
            println!("Streaming is {:.2}x faster than naive.", speedup);
        }

        if let (Some(batched), Some(streaming)) = (batched, streaming) {
            let speedup = batched.total_time_ms as f64 / streaming.total_time_ms as f64;
            println!("Streaming is {:.2}x faster than batched.\n", speedup);
        }

        if let (Some(naive), Some(batched)) = (naive, batched) {
            let ratio = batched.throughput / naive.throughput;
            println!("Batched throughput is {:.2}x higher than naive.", ratio);
        }

        if let (Some(naive), Some(streaming)) = (naive, streaming) {
            let ratio = streaming.throughput / naive.throughput;
            println!("Streaming throughput is {:.2}x higher than naive.", ratio);
        }

        if let (Some(batched), Some(streaming)) = (batched, streaming) {
            let ratio = streaming.throughput / batched.throughput;
            println!(
                "Streaming throughput is {:.2}x higher than batched.\n",
                ratio
            );
        }

        if let (Some(naive), Some(batched)) = (naive, batched) {
            let ratio = batched.peak_memory_mb as f64 / naive.peak_memory_mb as f64;
            println!("Batched peak memory is {:.2}x higher than naive.", ratio);
        }

        if let (Some(naive), Some(streaming)) = (naive, streaming) {
            let ratio = streaming.peak_memory_mb as f64 / naive.peak_memory_mb as f64;
            println!("Streaming peak memory is {:.2}x higher than naive.", ratio);
        }

        if let (Some(batched), Some(streaming)) = (batched, streaming) {
            let ratio = streaming.peak_memory_mb as f64 / batched.peak_memory_mb as f64;
            println!(
                "Streaming peak memory is {:.2}x higher than batched.\n",
                ratio
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn saves_csv() {
        let mut collector = MetricsCollector::new();

        collector.add_run(ProcessingRun::new("naive", 100, 15000, 450, 230, 290));
        collector.add_run(ProcessingRun::new("batched", 100, 8000, 180, 220, 285));

        let path = Path::new("test_metrics.csv");
        collector.save_csv(path).unwrap();

        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("naive"));
        assert!(contents.contains("batched"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn prints_comparison() {
        let mut collector = MetricsCollector::new();

        collector.add_run(ProcessingRun::new("naive", 100, 15234, 450, 230, 290));
        collector.add_run(ProcessingRun::new("batched", 100, 8456, 180, 220, 285));
        collector.add_run(ProcessingRun::new("streaming", 100, 5123, 120, 215, 280));

        collector.print_comparison();
    }
}
