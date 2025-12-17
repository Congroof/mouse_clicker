use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT, SendInput,
};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum HotKeyRegisterError {
    #[error("hot key {0} already registered")]
    DuplicateRegister(i32),
    #[error("hot key {0} not registered")]
    NotRegistered(i32),
    #[error("internal error")]
    InternalError,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClickType {
    Left,
    Right,
    Middle,
}

pub enum HotkeyEvent {
    HotKeyPressed { id: i32 },
    Register { id: i32, click_type: ClickType },
    Unregister { id: i32 },
    ConfigChange { times: usize, duration: u64 },
    ManualToggle { id: i32, start: bool },
    TaskCompleted { id: i32 },
}

pub type EventReceiver = tokio::sync::mpsc::UnboundedReceiver<HotkeyEvent>;
pub type EventSender = tokio::sync::mpsc::UnboundedSender<HotkeyEvent>;

// 状态通知回调
pub type StatusCallback = Arc<dyn Fn(bool) + Send + Sync>;

struct HotkeyInfo {
    click_type: ClickType,
    status: bool,
}

struct Config {
    times: usize,
    duration: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            times: 0,
            duration: 50,
        }
    }
}

pub struct KeyboardEventHandler {
    hotkey_map: HashMap<i32, HotkeyInfo>,
    oneshot_map: HashMap<i32, tokio::sync::oneshot::Sender<bool>>,
    config: Config,
    status_callback: Option<StatusCallback>,
    event_sender: Option<EventSender>,
}

impl KeyboardEventHandler {
    pub fn new() -> Self {
        Self {
            hotkey_map: std::collections::HashMap::new(),
            oneshot_map: std::collections::HashMap::new(),
            config: Config::default(),
            status_callback: None,
            event_sender: None,
        }
    }

    pub fn with_status_callback(mut self, callback: StatusCallback) -> Self {
        self.status_callback = Some(callback);
        self
    }

    pub fn with_event_sender(mut self, sender: EventSender) -> Self {
        self.event_sender = Some(sender);
        self
    }

    fn config_change(&mut self, times: usize, duration: u64) {
        self.config.times = times;
        self.config.duration = duration;
    }

    fn oneshot_clean(&mut self, id: i32) {
        if let Some(tx) = self.oneshot_map.remove(&id) {
            let _ = tx.send(true);
        }
    }

    fn handle_hotkey_pressed(&mut self, id: i32) {
        if let Some(hotkey_info) = self.hotkey_map.get_mut(&id) {
            hotkey_info.status = !hotkey_info.status;
            let new_status = hotkey_info.status;
            let click_type = hotkey_info.click_type.clone();

            if let Some(callback) = &self.status_callback {
                callback(new_status);
            }

            if new_status {
                self.start_clicking(id, click_type);
            } else {
                self.oneshot_clean(id);
            }
        }
    }

    fn handle_manual_toggle(&mut self, id: i32, start: bool) {
        if let Some(hotkey_info) = self.hotkey_map.get_mut(&id) {
            if hotkey_info.status == start {
                return;
            }

            hotkey_info.status = start;
            let click_type = hotkey_info.click_type.clone();

            if let Some(callback) = &self.status_callback {
                callback(start);
            }

            if start {
                self.start_clicking(id, click_type);
            } else {
                self.oneshot_clean(id);
            }
        }
    }

    fn start_clicking(&mut self, id: i32, click_type: ClickType) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let times = self.config.times;
        let duration = self.config.duration;
        let status_callback = self.status_callback.clone();
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            let completed = tokio::select! {
                _ = rx => false,
                _ = mouse_click(click_type, duration, times) => true,
            };

            if completed {
                if let Some(callback) = status_callback {
                    callback(false);
                }
                if let Some(sender) = event_sender {
                    let _ = sender.send(HotkeyEvent::TaskCompleted { id });
                }
            }
        });
        self.oneshot_map.insert(id, tx);
    }

    // 异步处理键盘事件
    pub async fn run(&mut self, mut evt_rx: EventReceiver) {
        while let Some(event) = evt_rx.recv().await {
            match event {
                HotkeyEvent::Register { id, click_type } => {
                    self.hotkey_map.insert(
                        id,
                        HotkeyInfo {
                            click_type,
                            status: false,
                        },
                    );
                }
                HotkeyEvent::Unregister { id } => {
                    self.hotkey_map.remove(&id);
                    self.oneshot_clean(id);
                }
                HotkeyEvent::HotKeyPressed { id } => {
                    self.handle_hotkey_pressed(id);
                }
                HotkeyEvent::ConfigChange { times, duration } => {
                    self.config_change(times, duration);
                }
                HotkeyEvent::ManualToggle { id, start } => {
                    self.handle_manual_toggle(id, start);
                }
                HotkeyEvent::TaskCompleted { id } => {
                    self.oneshot_map.remove(&id);
                    if let Some(hotkey_info) = self.hotkey_map.get_mut(&id) {
                        hotkey_info.status = false;
                    }
                }
            }
        }
    }
}

fn send_mouse_click(click_type: &ClickType) {
    let (down, up) = match click_type {
        ClickType::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        ClickType::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        ClickType::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    };

    let inputs = [
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: down,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: up,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];

    unsafe {
        let sent = SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
        if sent != inputs.len() as u32 {
            eprintln!("SendInput failed, sent {sent} of {}", inputs.len());
        }
    }
}

async fn mouse_click(click_type: ClickType, duration_ms: u64, times: usize) {
    let mut remaining = times;
    loop {
        send_mouse_click(&click_type);

        if times > 0 {
            if remaining == 1 {
                break;
            }
            remaining -= 1;
        }

        tokio::time::sleep(std::time::Duration::from_millis(duration_ms)).await;
    }
}
