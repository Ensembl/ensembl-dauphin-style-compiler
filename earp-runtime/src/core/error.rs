#[cfg_attr(debug_assertions,derive(Debug))]
pub enum EarpRuntimeError {
    BadEarpFile(String)
}