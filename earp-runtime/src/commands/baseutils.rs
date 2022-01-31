use std::{sync::Arc};

use crate::{runtime::{context::Context, operand::Operand, value::EarpValue}, core::error::EarpFault};

pub fn get_any<'a>(context: &'a Context, operand: &Operand) -> Result<Arc<Box<dyn EarpValue>>,EarpFault> {
    Ok(match operand {
        Operand::Register(index) => context.register_get(*index)?.clone(),
        Operand::UpRegister(index) => context.register_get_up(*index)?.clone(),
        Operand::String(value) => Arc::new(Box::new(value.clone())),
        Operand::Boolean(value) => Arc::new(Box::new(value.clone())),
        Operand::Integer(value) => Arc::new(Box::new(value.clone())),
        Operand::Float(value) => Arc::new(Box::new(value.clone())),
    })
}

pub fn set(context: &mut Context, operand: &Operand, value: Arc<Box<dyn EarpValue>>) -> Result<(),EarpFault> {
    match operand {
        Operand::Register(index) => context.register_set(*index,value),
        Operand::UpRegister(index) => context.register_set_up(*index,value),
        _ => Err(EarpFault(format!("can only assign to registers")))
    }
}
