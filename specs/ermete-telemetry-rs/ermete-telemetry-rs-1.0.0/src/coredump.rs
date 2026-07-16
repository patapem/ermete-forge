pub struct CoredumpWatcher {}

impl CoredumpWatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn poll_new_crashes(&mut self) -> Option<String> {
        // Placeholder for polling /var/lib/systemd/coredump
        None
    }
}
