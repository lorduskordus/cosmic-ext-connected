//! `SmsConversationStore` — owns SMS conversation state, message caches,
//! subscription orchestration, optimistic-send state, contacts, and SMS
//! notification dedup.
//!
//! Skeleton only: types and method signatures defined per the C10 Spike
//! Findings type-signature sketch. Method bodies are stubbed and nothing
//! in `app.rs` references this module yet.

#![allow(dead_code)] // M1 skeleton; first call sites land in M2

use crate::app::{DeviceInfo, Message};
use crate::config::Config;
use cosmic::iced::Subscription;
use cosmic::Element;
use kdeconnect_dbus::plugins::SmsMessage;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::Connection;

/// Read-only context the parent app passes to the store on each call.
pub struct SmsCtx<'a> {
    pub conn: &'a Arc<Mutex<Connection>>,
    pub config: &'a Config,
    pub devices: &'a [DeviceInfo],
}

/// Reply from the store back to the parent app describing app-level
/// state changes the caller must apply.
#[derive(Debug)]
pub enum SmsReply {
    /// SMS view is closing — caller should reset `view_mode` to `DevicePage`.
    ExitSms,
    /// Emit a transient status message via the app's normal status flow.
    Status(String),
    /// No app-level state change required.
    NoOp,
}

/// Which SMS sub-view the parent app is rendering.
#[derive(Debug, Clone, Copy)]
pub enum SmsViewMode {
    ConversationList,
    MessageThread,
    NewMessage,
}

pub struct SmsConversationStore;

impl SmsConversationStore {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &mut self,
        _msg: Message,
        _ctx: &SmsCtx,
    ) -> (cosmic::app::Task<Message>, SmsReply) {
        unimplemented!()
    }

    pub fn view(&self, _mode: SmsViewMode) -> Element<'_, Message> {
        unimplemented!()
    }

    pub fn subscriptions(&self, _config: &Config) -> Vec<Subscription<Message>> {
        unimplemented!()
    }

    pub fn open(
        &mut self,
        _device_id: String,
        _device_name: Option<String>,
        _ctx: &SmsCtx,
    ) -> cosmic::app::Task<Message> {
        unimplemented!()
    }

    pub fn close(&mut self) {
        unimplemented!()
    }

    pub fn handle_notification(
        &mut self,
        _device_id: String,
        _message: SmsMessage,
        _ctx: &SmsCtx,
    ) -> cosmic::app::Task<Message> {
        unimplemented!()
    }
}

impl Default for SmsConversationStore {
    fn default() -> Self {
        Self::new()
    }
}
