use crate::core::error::{EarpFault, EarpError};

use super::{command::Command, operand::Operand, context::Context};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Instruction {
    command: Command,
    operands: Vec<Operand>
}

impl Instruction {
    pub fn new(command: &Command, operands: &[Operand]) -> Result<Instruction,EarpError> {
        command.check_spec(operands)?;
        Ok(Instruction { command: command.clone(), operands: operands.to_vec() })
    }

    fn call(&self, context: &mut Context) -> Result<(),EarpFault> {
        self.command.call(context,&self.operands)
    }
}
