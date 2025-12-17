mod keyboard_event_handler;
mod message_loop;

use keyboard_event_handler::{EventReceiver, KeyboardEventHandler};
use message_loop::{Command, CommandReceiver, EventSender, MessageLoop};

pub use keyboard_event_handler::{ClickType, HotKeyRegisterError, StatusCallback};
pub use message_loop::CommandSender;

pub struct MouseClicker;

impl MouseClicker {
    pub async fn register_hotkey(
        cmd_tx: &CommandSender,
        id: i32,
        modifiers: u32,
        vk: u32,
        click_type: ClickType,
    ) -> Result<(), HotKeyRegisterError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let command = Command::Register {
            id,
            modifiers,
            vk,
            click_type,
            tx,
        };
        let _ = cmd_tx.send(command).await;
        let ok = rx.await.map_err(|_| HotKeyRegisterError::InternalError)?;
        if !ok {
            return Err(HotKeyRegisterError::DuplicateRegister(id));
        }
        Ok(())
    }

    pub async fn unregister_hotkey(
        cmd_tx: &CommandSender,
        id: i32,
    ) -> Result<(), HotKeyRegisterError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let command = Command::Unregister { id, tx };
        let _ = cmd_tx.send(command).await;
        let ok = rx.await.map_err(|_| HotKeyRegisterError::InternalError)?;
        if !ok {
            return Err(HotKeyRegisterError::NotRegistered(id));
        }
        Ok(())
    }

    pub async fn config_change(
        cmd_tx: &CommandSender,
        times: usize,
        duration: u64,
    ) -> Result<(), HotKeyRegisterError> {
        let command = Command::ConfigChange { times, duration };
        cmd_tx
            .send(command)
            .await
            .map_err(|_| HotKeyRegisterError::InternalError)?;
        Ok(())
    }

    pub async fn manual_toggle(
        cmd_tx: &CommandSender,
        id: i32,
        start: bool,
    ) -> Result<(), HotKeyRegisterError> {
        let command = Command::ManualToggle { id, start };
        cmd_tx
            .send(command)
            .await
            .map_err(|_| HotKeyRegisterError::InternalError)?;
        Ok(())
    }

    pub fn event_loop_task(cmd_rx: CommandReceiver, evt_tx: EventSender) {
        MessageLoop::start(cmd_rx, evt_tx);
    }

    pub async fn keyboard_event_handler_task(
        evt_rx: EventReceiver,
        evt_tx: EventSender,
        status_callback: Option<StatusCallback>,
    ) {
        let mut handler = KeyboardEventHandler::new()
            .with_event_sender(evt_tx);
        if let Some(callback) = status_callback {
            handler = handler.with_status_callback(callback);
        }
        handler.run(evt_rx).await;
    }
}
