use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction,
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use serde_cbor::Value as CborValue;

pub struct AddStickAuthorityCommand(Register);

impl Command for AddStickAuthorityCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize()]))
    }
}

pub struct AddStickAuthorityCommandType();

impl CommandType for AddStickAuthorityCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 1,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","add_stick_authority"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        Ok(Box::new(AddStickAuthorityCommand(it.regs[0])))
    }
}
