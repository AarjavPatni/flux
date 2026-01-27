// src/memory_monitor.rs

use sysinfo::{Pid, ProcessesToUpdate, System};

pub struct MemoryMonitor {
    system: System,
    pid: Pid,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        let system = System::new();
        let pid = sysinfo::get_current_pid().unwrap();
        MemoryMonitor { system, pid }
    }

    /// Get current process memory usage in MB
    pub fn current_usage_mb(&mut self) -> u64 {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        if let Some(process) = self.system.process(self.pid) {
            process.memory() / 1_024 / 1_024
        } else {
            0
        }
    }

    /// Get available memory in MB
    pub fn available_mb(&mut self) -> u64 {
        self.system.refresh_memory();
        self.system.available_memory() / 1_024 / 1_024
    }

    /// Get memory usage as percentage (0-100)
    pub fn usage_percent(&mut self) -> f32 {
        self.system.refresh_memory();
        let used_mem = self.system.used_memory() / 1_024 / 1_024;
        let total_mem = self.system.total_memory() / 1_024 / 1_024;

        (used_mem as f32 / total_mem as f32) * 100.0
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
