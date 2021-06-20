#[macro_export]
macro_rules! force_branch {
    ($enum:tt,$branch:tt,$var:ident) => {
        match $var { $enum::$branch(x) => Ok(x.clone()), _ => Err(Message::CodeInvariantFailed(format!("unexpected enum branch"))) }
    };
}