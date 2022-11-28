use crate::simple_interp_command;
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use std::str::from_utf8;

simple_interp_command!(SplitStringInterpCommand,SplitStringDeserializer,31,4,(0,1,2,3));
simple_interp_command!(BaseFlipInterpCommand,BaseFlipDeserializer,38,2,(0,1));

impl InterpCommand for BaseFlipInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let datas = registers.get_strings(&self.1)?;
        let mut out = vec![];
        for data in datas.iter() {
            out.push(match data.as_str() {
                "c" => "g", "C" => "G",
                "g" => "c", "G" => "C",
                "a" => "t", "A" => "T",
                "t" => "a", "T" => "A",
                x => x
            }.to_string());
        }
        registers.write(&self.0,InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SplitStringInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let strings = registers.get_bytes(&self.3)?;
        let mut out_offset = vec![];
        let mut out_length = vec![];
        let mut out_data = vec![];
        for bytes in strings.iter() {
            out_offset.push(out_data.len());
            let mut more : Vec<_> = from_utf8(bytes)?.split("\0").map(|x| x.to_string()).collect();
            out_length.push(more.len());
            out_data.append(&mut more);

        }
        registers.write(&self.0,InterpValue::Strings(out_data));
        registers.write(&self.1,InterpValue::Indexes(out_offset));
        registers.write(&self.2,InterpValue::Indexes(out_length));
        Ok(CommandResult::SyncResult())
    }
}
