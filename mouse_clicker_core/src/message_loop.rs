use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
use windows_sys::Win32::UI::WindowsAndMessaging::{MSG, PM_REMOVE, PeekMessageW, WM_HOTKEY};

use super::keyboard_event_handler::HotkeyEvent;

#[derive(Clone)]
pub enum Command {
    Register { id: i32, modifiers: u32, vk: u32 },
    Unregister { id: i32 },
}

pub type CommandSender = tokio::sync::broadcast::Sender<Command>;
pub type CommandReceiver = tokio::sync::broadcast::Receiver<Command>;
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
                        Command::Register { id, modifiers, vk } => {
                            if 0 == RegisterHotKey(hwnd, id, modifiers, vk) {
                                eprintln!("RegisterHotKey error id={id}");
                            }
                        }
                        Command::Unregister { id } => {
                            if 0 == UnregisterHotKey(hwnd, id) {
                                eprintln!("UnregisterHotKey error id={id}");
                            }
                        }
                    }
                }

                let mut msg: MSG = std::mem::zeroed();
                if PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE) != 0 {
                    if msg.message == WM_HOTKEY {
                        let _ = evt_tx.send(msg.into());
                    }
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }
}
