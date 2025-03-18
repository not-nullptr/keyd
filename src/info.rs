use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{sync::RwLock, task, time};
use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
use windows::Win32::Media::Audio::{
    eConsole, eRender, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE,
    DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows_core::Interface;

struct AudioWrapper(IAudioMeterInformation);

pub struct InfoMonitor {
    pub time: Arc<RwLock<i64>>,
    pub level: Arc<RwLock<u32>>,
    pub loop_task: Option<task::JoinHandle<()>>,
    pub info: Arc<AudioWrapper>,
}

unsafe impl Send for AudioWrapper {}
unsafe impl Sync for AudioWrapper {}

impl InfoMonitor {
    pub fn new() -> Self {
        let info = unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok().unwrap();
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(
                &MMDeviceEnumerator,
                None,
                windows::Win32::System::Com::CLSCTX_ALL,
            )
            .unwrap();
            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .unwrap();
            device
                .Activate::<IAudioMeterInformation>(CLSCTX_ALL, None)
                .unwrap()
        };
        Self {
            time: Arc::new(RwLock::new(0)),
            level: Arc::new(RwLock::new(0)),
            loop_task: None,
            info: Arc::new(AudioWrapper(info)),
        }
    }

    pub fn begin_montioring(&mut self) {
        let time = Arc::clone(&self.time);
        let level = Arc::clone(&self.level);
        let info = Arc::clone(&self.info);
        self.loop_task = Some(task::spawn(async move {
            loop {
                time::sleep(Duration::from_millis(50)).await;
                // get unix time in seconds
                let unix_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                *time.write().await = unix_time;

                let new_level = unsafe { info.0.GetPeakValue().unwrap() };

                *level.write().await = (new_level * 100.0).powf(1.25) as u32;
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
