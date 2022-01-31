use super::{command::Command, operand::Operand, context::Context};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Instruction {
    command: Command,
    operands: Vec<Operand>
}

impl Instruction {
    pub fn new(command: &Command, operands: &[Operand]) -> Instruction {
        Instruction { command: command.clone(), operands: operands.to_vec() }
    }

    fn call(&self, context: &mut Context) {
        self.command.call(context,&self.operands);
    }
}
