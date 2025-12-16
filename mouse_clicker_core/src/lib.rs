mod keyboard_event_handler;
mod message_loop;

use keyboard_event_handler::{EventReceiver, KeyboardEventHandler};
use message_loop::{Command, CommandReceiver, EventSender, MessageLoop};

pub use keyboard_event_handler::HotKeyRegisterError;

use crate::message_loop::CommandSender;

pub struct MouseClicker;

impl MouseClicker {
    pub fn new_broadcast_channel(capacity: usize) -> (CommandSender, CommandReceiver) {
        let (cmd_tx, cmd_rx) = tokio::sync::broadcast::channel(capacity);
        (cmd_tx, cmd_rx)
    }

    pub fn register_hotkey(
        cmd_tx: &CommandSender,
        id: i32,
        modifiers: u32,
        vk: u32,
    ) -> Result<(), HotKeyRegisterError> {
        let _ = cmd_tx.send(Command::Register { id, modifiers, vk });
        // todo!("增加oneshot机制");
        Ok(())
    }

    pub fn unregister_hotkey(cmd_tx: &CommandSender, id: i32) -> Result<(), HotKeyRegisterError> {
        let _ = cmd_tx.send(Command::Unregister { id });
        // todo!("增加oneshot机制");
        Ok(())
    }

    pub fn event_loop_task(cmd_rx: CommandReceiver, evt_tx: EventSender) {
        MessageLoop::start(cmd_rx, evt_tx);
    }

    pub async fn keyboard_event_handler_task(cmd_rx: CommandReceiver, evt_rx: EventReceiver) {
        KeyboardEventHandler::new().run(cmd_rx, evt_rx).await;
    }
}
