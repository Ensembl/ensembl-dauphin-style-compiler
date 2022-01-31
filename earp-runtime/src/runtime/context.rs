use std::sync::Arc;

use crate::core::error::EarpFault;

use super::{stack::Stack, config::Config, value::EarpValue};

pub struct Context {
    pc: i64,
    stack: Stack
}

impl Context {
    pub fn new(config: &Config) -> Context {
        Context {
            stack: Stack::new(config),
            pc: 0
        }
    }

    pub fn halt(&mut self) {
        self.pc = -2;
    }

    pub fn register_get(&self, index: usize) -> Result<&Arc<Box<dyn EarpValue>>,EarpFault> {
        self.stack.get(index)
    }

    pub fn register_set(&mut self, index: usize, value: Arc<Box<dyn EarpValue>>) -> Result<(),EarpFault> {
        self.stack.set(index,value)
    }

    pub fn register_get_up(&self, index: usize) -> Result<&Arc<Box<dyn EarpValue>>,EarpFault> {
        self.stack.get_up(index)
    }

    pub fn register_set_up(&mut self, index: usize, value: Arc<Box<dyn EarpValue>>) -> Result<(),EarpFault> {
        self.stack.set_up(index,value)
    }
}
