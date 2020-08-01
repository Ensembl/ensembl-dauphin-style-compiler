use dauphin_compile::command::{ Command, CommandSchema, CommandType, CommandTrigger, CompLibRegister, Instruction, PreImagePrepare, PreImageOutcome };
use dauphin_interp::command::Identifier;
use dauphin_interp::runtime::Register;
use serde_cbor::Value as CborValue;
use dauphin_compile::model::PreImageContext;

pub struct LookupCommand(Register,Register,Register,Register,Register,Register);

impl Command for LookupCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),self.4.serialize(),self.5.serialize()]))
    }

    fn simple_preimage(&self, context: &mut PreImageContext) -> anyhow::Result<PreImagePrepare> { 
        Ok(if context.is_reg_valid(&self.1) && context.is_reg_valid(&self.2) && 
                context.is_reg_valid(&self.3) && context.is_reg_valid(&self.4) && 
                context.is_reg_valid(&self.5) && !context.is_last() {
            PreImagePrepare::Replace
        } else if let Some(a) = context.get_reg_size(&self.1) {
            PreImagePrepare::Keep(vec![(self.0.clone(),a)])
        } else {
            PreImagePrepare::Keep(vec![])
        })
    }

    fn preimage_post(&self, _context: &mut PreImageContext) -> anyhow::Result<PreImageOutcome> {
        Ok(PreImageOutcome::Constant(vec![self.0]))
    }
}

pub struct LookupCommandType();

impl CommandType for LookupCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 6,
            trigger: CommandTrigger::Command(Identifier::new("std","lookup"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        Ok(Box::new(LookupCommand(it.regs[0],it.regs[1],it.regs[2],it.regs[3],it.regs[4],it.regs[5])))
    }    
}

pub(super) fn library_map_commands(set: &mut CompLibRegister) {
    set.push("lookup",Some(3),LookupCommandType());
}
