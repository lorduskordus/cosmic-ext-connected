//! Logical conversation: wraps one-or-more underlying SMS thread IDs that
//! represent the same user-perceived conversation.
//!
//! At M6 every `LogicalConversation` is a 1:1 wrapper around a single
//! `ConversationSummary` (no merging). M7 introduces the reaction-bucket
//! merge heuristic via `merge_into_logical`, which collapses split threads
//! into a single `LogicalConversation` carrying multiple `merged_thread_ids`.

#![allow(dead_code)] // M7 lights up 'merged_thread_ids', 'subscription_id', 'unread_count'

use kdeconnect_dbus::plugins::ConversationSummary;

/// A user-perceived conversation, possibly composed of multiple underlying
/// SMS thread IDs that AOSP has split apart (e.g. by iOS reaction echoes
/// arriving with slightly-different address-sets).
#[derive(Debug, Clone)]
pub struct LogicalConversation {
    /// Thread ID used for `replyToConversation`. For a 1:1 wrapper this is
    /// the underlying thread's ID; once merging is active, the split-by-case
    /// reply rule picks the most-recently-active sibling within a symmetric
    /// merge group.
    pub primary_thread_id: i64,
    /// All underlying thread IDs composing this logical conversation.
    /// Always contains `primary_thread_id`. Single-element at M6.
    pub merged_thread_ids: Vec<i64>,
    /// SIM subscription ID. Merge never crosses subID boundaries, so all
    /// threads in `merged_thread_ids` share this value.
    pub subscription_id: i64,
    /// Union of every underlying thread's address-set, deduplicated.
    pub addresses: Vec<String>,
    /// Preview of the most recent message body across all merged threads.
    pub last_message_preview: String,
    /// Unix-millis timestamp of the most recent message across all merged threads.
    pub last_message_timestamp: i64,
    /// Whether the most recent message is an MMS with attachments.
    pub has_attachments: bool,
    /// Number of underlying threads currently flagged unread. At M6 this is
    /// `0` or `1` (1:1 wrapper); M7 sums across merged threads.
    pub unread_count: usize,
}

impl LogicalConversation {
    /// Wrap a single underlying conversation 1:1. M6's only construction path.
    pub fn from_single(cs: ConversationSummary) -> Self {
        Self {
            primary_thread_id: cs.thread_id,
            merged_thread_ids: vec![cs.thread_id],
            subscription_id: cs.sub_id,
            addresses: cs.addresses,
            last_message_preview: cs.last_message,
            last_message_timestamp: cs.timestamp,
            has_attachments: cs.has_attachments,
            unread_count: if cs.unread { 1 } else { 0 },
        }
    }
}