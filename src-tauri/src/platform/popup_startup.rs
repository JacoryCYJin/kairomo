use std::sync::Mutex;

/// The first open request waits for the frontend's complete initial frame.
#[derive(Default)]
pub struct PopupStartup(pub Mutex<Startup>);
#[derive(Default)]
pub struct Startup {
    ready: bool,
    requested: bool,
}
impl Startup {
    pub fn request(&mut self) -> bool {
        if self.ready {
            true
        } else {
            self.requested = true;
            false
        }
    }
    pub fn prepared(&mut self) -> bool {
        self.ready = true;
        std::mem::take(&mut self.requested)
    }
    pub fn cancel(&mut self) {
        self.requested = false;
    }
}
#[cfg(test)]
mod tests {
    use super::Startup;
    #[test]
    fn cold_open_waits_and_multiple_requests_coalesce() {
        let mut startup = Startup::default();
        assert!(!startup.request());
        assert!(!startup.request());
        assert!(startup.prepared());
        assert!(!startup.prepared());
        assert!(startup.request());
    }
    #[test]
    fn background_preparation_does_not_open_an_unrequested_window() {
        let mut startup = Startup::default();
        assert!(!startup.prepared());
        assert!(startup.request());
    }
    #[test]
    fn hiding_or_opening_details_cancels_pending_reveal() {
        let mut startup = Startup::default();
        assert!(!startup.request());
        startup.cancel();
        assert!(!startup.prepared());
        assert!(startup.request());
    }
}
