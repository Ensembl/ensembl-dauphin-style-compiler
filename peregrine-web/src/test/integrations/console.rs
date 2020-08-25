use crate::PgConsole;
use std::sync::{ Arc, Mutex };

#[derive(Clone)]
pub struct TestConsole(Arc<Mutex<Vec<String>>>);

impl TestConsole {
    pub fn new() -> TestConsole {
        TestConsole(Arc::new(Mutex::new(vec![])))
    }

    pub fn message(&self, msg: &str) {
        print!("console: {}\n",msg);
        self.0.lock().unwrap().push(msg.to_string());
    }

    pub fn take_all(&self) -> Vec<String> {
        self.0.lock().unwrap().drain(..).collect()
    }
}

impl PgConsole for TestConsole {
    fn warn(&self, msg: &str) {
        self.message(msg);
    }

    fn error(&self, msg: &str) {
        self.message(msg);
    }
}
