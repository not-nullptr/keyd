use std::{sync::Arc, time::Duration};

use sysinfo::System;
use tokio::{sync::RwLock, task, time};

pub struct InfoMonitor {
    pub cpu_usage: Arc<RwLock<u8>>,
    pub mem_usage: Arc<RwLock<u8>>,
    pub process_count: Arc<RwLock<u16>>,
    sys: Arc<RwLock<System>>,
    loop_task: Option<task::JoinHandle<()>>,
}

impl InfoMonitor {
    pub fn new() -> Self {
        Self {
            cpu_usage: Arc::new(RwLock::new(0)),
            mem_usage: Arc::new(RwLock::new(0)),
            process_count: Arc::new(RwLock::new(0)),
            loop_task: None,
            sys: Arc::new(RwLock::new(System::new_all())),
        }
    }

    pub fn begin_montioring(&mut self) {
        let cpu_usage = Arc::clone(&self.cpu_usage);
        let mem_usage = Arc::clone(&self.mem_usage);
        let process_count = Arc::clone(&self.process_count);
        let sys = Arc::clone(&self.sys);
        self.loop_task = Some(task::spawn(async move {
            loop {
                time::sleep(Duration::from_millis(200)).await;
                let mut sys = sys.write().await;
                sys.refresh_all();
                let mut cpu_usage = cpu_usage.write().await;
                let mut mem_usage = mem_usage.write().await;
                let mut process_count = process_count.write().await;
                *cpu_usage = sys.global_cpu_usage() as u8;
                *mem_usage = ((sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0) as u8;
                *process_count = sys.processes().len() as u16;
            }
        }));
    }
}

impl Drop for InfoMonitor {
    fn drop(&mut self) {
        if let Some(task) = self.loop_task.take() {
            task.abort();
        }
    }
}
