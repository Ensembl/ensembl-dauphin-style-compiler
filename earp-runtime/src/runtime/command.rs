use std::{sync::Arc, fmt::Debug};

use super::{context::Context, operand::Operand};

#[derive(Clone)]
pub struct Command {
    closure: Arc<Box<dyn Fn(&mut Context, &[Operand]) + 'static>>
}

#[cfg(debug_assertions)]
impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").field("closure", &"...").finish()
    }
}

impl Command {
    pub fn new<F>(closure: F) -> Command where F: Fn(&mut Context, &[Operand]) + 'static {
        Command {
            closure: Arc::new(Box::new(closure))
        }
    }

    pub(super) fn call(&self, context: &mut Context, operands: &[Operand]) {
        (self.closure)(context,operands)
    }
}
