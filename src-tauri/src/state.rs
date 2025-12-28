use std::sync::atomic::AtomicBool;

pub struct AppState {
    pub should_stop: AtomicBool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            should_stop: AtomicBool::new(false),
        }
    }
}