use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
use windows_sys::Win32::UI::WindowsAndMessaging::{MSG, PM_REMOVE, PeekMessageW, WM_HOTKEY};

use super::keyboard_event_handler::{ClickType, HotkeyEvent};

pub enum Command {
    Register {
        id: i32,
        modifiers: u32,
        vk: u32,
        click_type: ClickType,
        // 注册结果的sender
        tx: tokio::sync::oneshot::Sender<bool>,
    },
    Unregister {
        id: i32,
        tx: tokio::sync::oneshot::Sender<bool>,
    },
    ConfigChange {
        times: usize,
        duration: u64,
    },
    ManualToggle {
        id: i32,
        start: bool,
    },
}

pub type CommandSender = tokio::sync::mpsc::Sender<Command>;
pub type CommandReceiver = tokio::sync::mpsc::Receiver<Command>;
pub type EventSender = tokio::sync::mpsc::UnboundedSender<HotkeyEvent>;

pub struct MessageLoop;

impl MessageLoop {
    // 需要在单独的线程中同时处理command和event
    pub fn start(mut cmd_rx: CommandReceiver, evt_tx: EventSender) {
        unsafe {
            let hwnd = std::ptr::null_mut();
            loop {
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        Command::Register {
                            id,
                            click_type,
                            modifiers,
                            vk,
                            tx,
                        } => {
                            let result = RegisterHotKey(hwnd, id, modifiers, vk) != 0;
                            let _ = tx.send(result);
                            if result {
                                let _ = evt_tx.send(HotkeyEvent::Register { id, click_type });
                            }
                        }
                        Command::Unregister { id, tx } => {
                            let result = UnregisterHotKey(hwnd, id) != 0;
                            let _ = tx.send(result);
                            if result {
                                let _ = evt_tx.send(HotkeyEvent::Unregister { id });
                            }
                        }
                        Command::ConfigChange { times, duration } => {
                            let _ = evt_tx.send(HotkeyEvent::ConfigChange { times, duration });
                        }
                        Command::ManualToggle { id, start } => {
                            let _ = evt_tx.send(HotkeyEvent::ManualToggle { id, start });
                        }
                    }
                }

                let mut msg: MSG = std::mem::zeroed();
                if PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE) != 0 {
                    if msg.message == WM_HOTKEY {
                        let _ = evt_tx.send(HotkeyEvent::HotKeyPressed {
                            id: msg.wParam as i32,
                        });
                    }
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }
}
