pub trait PgConsole {
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
}
