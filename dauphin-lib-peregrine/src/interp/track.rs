use crate::simple_interp_command;
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;

simple_interp_command!(AppendGroupInterpCommand,AppendGroupDeserializer,47,3,(0,1,2));
simple_interp_command!(AppendDepthInterpCommand,AppendDepthDeserializer,48,3,(0,1,2));

impl InterpCommand for AppendGroupInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let allotments = registers.get_strings(&self.1)?.to_vec();
        let groups = registers.get_strings(&self.2)?.to_vec();
        let mut out = vec![];
        for (allotment,group) in allotments.iter().zip(groups.iter().cycle()) {
            if allotment.is_empty() {
                out.push("".to_string());
            } else {
                out.push(format!("{}/{}",allotment,group));
            }
        }
        registers.write(&self.0, InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for AppendDepthInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let allotments = registers.get_strings(&self.1)?.to_vec();
        let depths = registers.get_numbers(&self.2)?.to_vec();
        let mut out = vec![];
        for (allotment,depth) in allotments.iter().zip(depths.iter().cycle()) {
            if allotment.is_empty() {
                out.push("".to_string());
            } else {
                out.push(format!("{}[{}]",allotment,depth));
            }
        }
        registers.write(&self.0, InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}
