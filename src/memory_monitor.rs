// src/memory_monitor.rs

use sysinfo::System;

pub struct MemoryMonitor {
    system: System,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        todo!()
    }

    /// Get current memory usage in MB
    pub fn current_usage_mb(&mut self) -> u64 {
        todo!()
        // Hint: self.system.refresh_memory();
        // Then: self.system.used_memory() / 1_024 / 1_024
    }

    /// Get available memory in MB
    pub fn available_mb(&mut self) -> u64 {
        todo!()
    }

    /// Get memory usage as percentage (0-100)
    pub fn usage_percent(&mut self) -> f32 {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_memory() {
        let mut monitor = MemoryMonitor::new();
        let usage = monitor.current_usage_mb();
        assert!(usage > 0);
        assert!(usage < 1_000_000); // Less than 1TB :)
    }

    #[test]
    fn reports_percentage() {
        let mut monitor = MemoryMonitor::new();
        let percent = monitor.usage_percent();
        assert!(percent > 0.0);
        assert!(percent <= 100.0);
    }
}
