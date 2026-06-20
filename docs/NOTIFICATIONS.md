# Notification Systems

Desktop notification implementations for SMS, calls, and file transfers.

## SMS Notifications

Shows desktop notifications when new SMS messages are received.

### Implementation

1. **D-Bus Signal**: Separate subscription listens for `conversationUpdated` signals from `org.kde.kdeconnect.device.conversations`

2. **Message Filtering**: Only incoming messages notified (MessageType::Inbox)

3. **Deduplication**: `last_seen_sms: HashMap<i64, i64>` tracks latest timestamp per thread_id

4. **Contact Resolution**: Sender names resolved via `ContactLookup` using synced vCards

5. **Privacy Settings**:
   - `sms_notifications` - Master toggle
   - `sms_notification_show_sender` - Show/hide sender name
   - `sms_notification_show_content` - Show/hide message preview

### Display

```rust
notify_rust::Notification::new()
    .summary(&summary)  // "New SMS" or "New SMS from {name}"
    .body(&body)        // Message content or "Message received"
    .icon("phone-symbolic")
    .appname("Connected")
    .show()
```

### Subscription Lifecycle

Active when:
- `config.sms_notifications` is enabled
- At least one device is both reachable AND paired

Auto-reconnects on D-Bus disconnection.

## Call Notifications

Shows notifications for incoming and missed phone calls.

### D-Bus Signal

The telephony plugin emits `callReceived` signal with:
- `event` - "callReceived" or "missedCall"
- `phone_number` - Caller's phone number
- `contact_name` - Contact name if available

### Privacy Settings

- `call_notifications` - Master toggle
- `call_notification_show_name` - Show/hide contact name
- `call_notification_show_number` - Show/hide phone number

### Display

```rust
notify_rust::Notification::new()
    .summary(&summary)  // "Incoming Call" or "Incoming call from {name}"
    .body(&device_name) // Which device received the call
    .icon("call-start-symbolic")  // or "call-missed-symbolic"
    .appname("Connected")
    .urgency(notify_rust::Urgency::Critical)
    .show()
```

### Limitation: Mute Ringer

KDE Connect handles ringer muting internally via KNotification. No D-Bus method exposed for external muting - would require upstream changes.

## File Receive Notifications

Shows notifications when files are received from connected devices.

### D-Bus Signal

Share plugin emits `shareReceived` signal with:
- `file_url` - file:// URL of received file

### Privacy Settings

- `file_notifications` - Master toggle

### Display

```rust
let mut notification = notify_rust::Notification::new();
notification
    .summary(&fl!("file-received-from", device = device_name))
    .body(&file_name)
    .icon("folder-download-symbolic")
    .appname("Connected")
    .timeout(notify_rust::Timeout::Milliseconds(NORMAL_NOTIFICATION_TIMEOUT_MS));
// .show() blocks (zbus), so run it on a blocking thread; log the result
// rather than discarding it (a bare `let _ = …show()` silently swallows
// errors/panics — the cause of a long "missing SMS toast" hunt in v0.6.0).
match tokio::task::spawn_blocking(move || notification.show()).await {
    Ok(Ok(_handle)) => tracing::debug!("File notification shown"),
    Ok(Err(e)) => tracing::warn!("Failed to show file notification: {}", e),
    Err(e) => tracing::warn!("File notification task panicked: {}", e),
}
```

### Timeout Handling

COSMIC's notification daemon **clamps** the freedesktop `expire_timeout` hint — it does **not** ignore it. Displayed duration = `min(requested, daemon cap)`, where the cap is `max_timeout_normal` (default 5000 ms) for normal/low urgency; urgent notifications are uncapped (`max_timeout_urgent = None`).

Connected requests a large bounded timeout and **defers the real duration to COSMIC** (so a future raised cap is honored automatically, and the default is 5 s today). There is no in-app duration setting and no manual close:

- **Normal toasts** (SMS / file / missed-call) request `NORMAL_NOTIFICATION_TIMEOUT_MS` (30 s) → clamped to the daemon's normal cap (5 s by default).
- **The critical incoming-call toast** requests `CALL_RING_TIMEOUT_MS` (30 s), which is the *literal* on-screen time since urgent notifications are uncapped — a distinct mechanism from the normal path despite the equal number. (Not `Timeout::Never`: Connected has no active dismissal on call-end, so `Never` would leave a stale toast.)

Both constants live in `constants.rs::notifications`. The prior model — a user-configured `notification_timeout_secs` slider plus a `Timeout::Never` + `show_and_auto_close()` manual-close workaround built on the mistaken belief that the daemon ignored `expire_timeout` — was **removed in v0.6.0**.

> Note: every notification site logs the `.show()` result (a `match`, not `let _ = …`). The success arm is `debug!` (suppressed in release, which defaults to the `warn` filter — see `main.rs`); failure/panic arms are `warn!` so they surface in release builds.

## Cross-Process Deduplication

COSMIC spawns multiple applet processes. Each independently receives D-Bus signals, causing duplicate desktop notifications. Traditional in-process deduplication doesn't work across processes.

### Solution: File-Based Locking

Each notification type uses a dedicated temp file with POSIX file locking:

| Type | Dedup file | Key format |
|------|-----------|------------|
| File | `/tmp/cosmic-connected-file-dedup` | `{file_url}` |
| SMS | `/tmp/cosmic-connected-sms-dedup` | `{thread_id}:{message_date}` |
| Call | `/tmp/cosmic-connected-call-dedup` | `{event}:{phone_number}` |

All use the same generic `should_show_notification()` function in `notifications.rs`:

```rust
fn should_show_notification(dedup_file: &str, key: &str) -> bool {
    // Open dedup file
    // Acquire exclusive lock with flock(fd, LOCK_EX)
    // Check if same key within 2 second window
    // Update file with new key and timestamp
    // Release lock with flock(fd, LOCK_UN)
}
```

Key points:
- Uses `libc::flock()` for atomic locking across processes
- 2-second deduplication window
- Static variables NOT shared between applet instances
- Call dedup key includes event type so `callReceived` and `missedCall` for the same number are treated as distinct notifications

## Unsupported: Incoming Ping Notifications

KDE Connect's ping plugin (`kdeconnect_ping`) does not emit D-Bus signals for incoming pings. When a ping is received, `kdeconnectd` handles it internally and sends a desktop notification directly via `KNotification`, bypassing any D-Bus signal mechanism. The applet cannot detect or replace incoming ping notifications. The ping plugin only exposes `sendPing()` methods (outgoing), not incoming signals.
