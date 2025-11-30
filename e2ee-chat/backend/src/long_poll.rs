use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct LongPollManager {
    waiters: HashMap<String, Vec<Arc<Notify>>>,
}

impl LongPollManager {
    pub fn new() -> Self {
        Self {
            waiters: HashMap::new(),
        }
    }

    pub fn add_waiter(&mut self, username: &str) -> Arc<Notify> {
        let notify = Arc::new(Notify::new());
        self.waiters
            .entry(username.to_string())
            .or_insert_with(Vec::new)
            .push(Arc::clone(&notify));
        notify
    }

    pub fn notify(&mut self, username: &str) {
        if let Some(waiters) = self.waiters.remove(username) {
            for waiter in waiters {
                waiter.notify_one();
            }
        }
    }
}
