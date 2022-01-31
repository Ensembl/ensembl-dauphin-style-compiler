use std::{sync::Arc, fmt::Debug};

use crate::core::error::{EarpFault, EarpError};

use super::{context::Context, operand::{Operand}};

#[derive(Clone)]
pub enum OperandSpec {
    Any,
    Register
}

impl OperandSpec {
    fn check_spec(&self, operand: &Operand) -> Result<(),EarpError> {
        match (self,operand) {
            (OperandSpec::Any,_) => Ok(()),
            (OperandSpec::Register,Operand::Register(_)) => Ok(()),
            (OperandSpec::Register,Operand::UpRegister(_)) => Ok(()),
            _ => Err(EarpError::BadOpcode(format!("instruction needed register, got constant")))
        }
    }
}

#[derive(Clone)]
pub struct CommandSpec {
    pub operand_spec: Vec<OperandSpec>
}

impl CommandSpec {
    fn check_spec(&self, operands: &[Operand]) -> Result<(),EarpError> {
        if self.operand_spec.len() != operands.len() {
            return Err(EarpError::BadOpcode(format!("instruction had wrong number of arguments")));
        }
        for (operand,spec) in operands.iter().zip(self.operand_spec.iter()) {
            spec.check_spec(operand)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Command {
    closure: Arc<Box<dyn Fn(&mut Context, &[Operand]) -> Result<(),EarpFault> + 'static>>,
    spec: CommandSpec
}

#[cfg(debug_assertions)]
impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").field("closure", &"...").finish()
    }
}

impl Command {
    pub fn new<F>(closure: F, spec: CommandSpec) -> Command where F: Fn(&mut Context, &[Operand]) -> Result<(),EarpFault> + 'static {
        Command {
            closure: Arc::new(Box::new(closure)),
            spec
        }
    }

    pub(super) fn check_spec(&self, operands: &[Operand]) -> Result<(),EarpError> {
        self.spec.check_spec(operands)
    }

    pub(super) fn call(&self, context: &mut Context, operands: &[Operand]) -> Result<(),EarpFault> {
        (self.closure)(context,operands)
    }
}
