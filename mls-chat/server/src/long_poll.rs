use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct LongPollManager {
    waiters: HashMap<String, Arc<Notify>>,
}

impl LongPollManager {
    pub fn new() -> Self {
        Self {
            waiters: HashMap::new(),
        }
    }

    pub fn add_waiter(&mut self, username: &str) -> Arc<Notify> {
        let notify = Arc::new(Notify::new());
        self.waiters.insert(username.to_string(), notify.clone());
        notify
    }

    pub fn notify(&mut self, username: &str) {
        if let Some(notify) = self.waiters.remove(username) {
            notify.notify_one();
        }
    }

    pub fn notify_all(&mut self, usernames: &[String]) {
        for username in usernames {
            self.notify(username);
        }
    }
}

impl Default for LongPollManager {
    fn default() -> Self {
        Self::new()
    }
}
