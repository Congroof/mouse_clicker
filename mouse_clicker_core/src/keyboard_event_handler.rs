use super::message_loop::{Command, CommandReceiver};
use std::collections::HashMap;
use windows_sys::Win32::UI::WindowsAndMessaging::MSG;

#[derive(Debug, thiserror::Error)]
pub enum HotKeyRegisterError {
    #[error("hot key {0} already registered")]
    DuplicateRegister(usize),
    #[error("hot key {0} not registered")]
    NotRegistered(usize),
}

pub struct HotkeyEvent {
    id: usize,
    wparam: usize,
    lparam: isize,
}

impl From<MSG> for HotkeyEvent {
    fn from(msg: MSG) -> Self {
        Self {
            id: msg.wParam,
            wparam: msg.wParam,
            lparam: msg.lParam,
        }
    }
}

// TODO: 实现鼠标点击功能
async fn mouse_click(duration: u64) {
    loop {
        println!("mouse click");
        tokio::time::sleep(std::time::Duration::from_secs(duration)).await;
    }
}

pub type EventReceiver = tokio::sync::mpsc::UnboundedReceiver<HotkeyEvent>;

pub struct KeyboardEventHandler {
    status_map: HashMap<usize, bool>,
    oneshot_map: HashMap<usize, tokio::sync::oneshot::Sender<bool>>,
}

impl KeyboardEventHandler {
    pub fn new() -> Self {
        Self {
            status_map: std::collections::HashMap::new(),
            oneshot_map: std::collections::HashMap::new(),
        }
    }

    fn handle_hotkey_event(&mut self, event: HotkeyEvent) {
        let entry = self.status_map.entry(event.id).or_insert(false);
        *entry = !*entry;
        if *entry {
            let (tx, rx) = tokio::sync::oneshot::channel();
            tokio::spawn(async {
                tokio::select! {
                    _ = rx => {
                        println!("mouse click task finished");
                    }
                    _ = mouse_click(1) => {}
                }
            });
            self.oneshot_map.insert(event.id, tx);
        } else {
            if let Some(tx) = self.oneshot_map.remove(&event.id) {
                let _ = tx.send(true);
            }
        }
    }

    // 异步处理键盘事件
    pub async fn run(&mut self, mut cmd_rx: CommandReceiver, mut evt_rx: EventReceiver) {
        loop {
            tokio::select! {
                event = evt_rx.recv() => {
                    match event {
                        Some(event) => {
                            self.handle_hotkey_event(event);
                        }
                        None => {
                            println!("evt_rx.recv() break");
                            break;
                        }
                    }
                }
                cmd = cmd_rx.recv() => {
                    if let Ok(cmd) = cmd {
                        match cmd {
                            Command::Register { id, .. } => {
                                let id = id as usize;
                                let _ = self.status_map.insert(id, false);
                            }
                            Command::Unregister { id } => {
                                let id = id as usize;
                                let _ = self.status_map.remove(&id);
                                let _ = self.oneshot_map.remove(&id);
                            }
                        }
                    } else {
                        println!("cmd_rx.recv() break");
                        break;
                    }
                }
            }
        }
    }
}
