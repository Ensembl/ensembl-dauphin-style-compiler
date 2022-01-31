use std::{sync::Arc};

use crate::core::error::EarpFault;

use super::{config::Config, value::EarpValue};

struct Frame {
    registers: Vec<Arc<Box<dyn EarpValue>>>
}

impl Frame {
    fn new() -> Frame {
        Frame {
            registers: vec![]
        }
    }
 
    fn get(&self, index: usize) -> Result<&Arc<Box<dyn EarpValue>>,EarpFault> {        
        self.registers.get(index).ok_or_else(||
            EarpFault(format!("register out of range: {}",index))
        )
    }

    fn set(&mut self, index: usize, value: Arc<Box<dyn EarpValue>>, reg_count: &mut usize, max_regs: usize) -> Result<(),EarpFault> {
        while self.registers.len() <= index {
            if *reg_count == max_regs {
                return Err(EarpFault(format!("too many registers")));
            }
            *reg_count += 1;
            self.registers.push(Arc::new(Box::new(())));
        }
        self.registers[index] = value;
        Ok(())
    }
}

pub struct Stack {
    stack: Vec<Frame>,
    register_count: usize,
    max_depth: usize,
    max_registers: usize
}

impl Stack {
    pub fn new(config: &Config) -> Stack {
        Stack {
            stack: vec![Frame::new(),Frame::new()],
            max_depth: config.max_stack_height,
            max_registers: config.max_registers,
            register_count: 0
        }
    }

    pub fn push(&mut self) -> Result<(),EarpFault> {
        if self.stack.len() >= self.max_depth {
            return Err(EarpFault(format!("stack overflow")));
        }
        self.stack.push(Frame::new());
        Ok(())
    }

    pub fn pop(&mut self) -> Result<(),EarpFault> {
        if self.stack.len() == 2 {
            return Err(EarpFault(format!("stack underflow")));
        }
        self.stack.pop();
        Ok(())
    }

    pub fn get(&self, index: usize) -> Result<&Arc<Box<dyn EarpValue>>,EarpFault> {
        self.stack.last().unwrap().get(index)
    }

    pub fn set(&mut self, index: usize, value: Arc<Box<dyn EarpValue>>) -> Result<(),EarpFault> {
        let (stack,register_count) = (&mut self.stack, &mut self.register_count);
       stack.last_mut().unwrap().set(index,value,register_count,self.max_registers)
    }

    pub fn get_up(&self, index: usize) -> Result<&Arc<Box<dyn EarpValue>>,EarpFault> {
        self.stack[self.stack.len()-2].get(index)
    }

    pub fn set_up(&mut self, index: usize, value: Arc<Box<dyn EarpValue>>) -> Result<(),EarpFault> {
        let (stack,register_count) = (&mut self.stack, &mut self.register_count);
        let len = stack.len();
        stack[len-2].set(index,value,register_count,self.max_registers)
    }
}
