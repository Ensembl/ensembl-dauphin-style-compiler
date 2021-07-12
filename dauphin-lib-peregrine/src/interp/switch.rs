use crate::simple_interp_command;
use peregrine_data::{ Channel, AllotmentRequest, ShapeRequest };
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult, AsyncBlock };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use crate::util::{ get_instance, get_peregrine };

simple_interp_command!(GetSwitchInterpCommand,GetSwitchDeserializer,32,4,(0,1,2,3));

impl InterpCommand for GetSwitchInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let path_data = registers.get_strings(&self.1)?.to_vec();
        let path_offset = registers.get_indexes(&self.2)?.to_vec();
        let path_length = registers.get_indexes(&self.3)?.to_vec();
        drop(registers);
        let request = get_instance::<ShapeRequest>(context,"request")?;
        let config = request.track();
        let mut out = vec![];
        for (offset,length) in path_offset.iter().zip(path_length.iter().cycle()) {
            let path = &path_data[*offset..(*offset+*length)].iter().map(|x| x.as_str()).collect::<Vec<_>>();
            out.push(config.get(path).is_some());
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Boolean(out));
        Ok(CommandResult::SyncResult())
    }
}
