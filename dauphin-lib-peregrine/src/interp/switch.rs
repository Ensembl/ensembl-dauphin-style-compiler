use crate::simple_interp_command;
use peregrine_data::ShapeRequest;
use peregrine_toolkit::eachorevery::eoestruct::{StructBuilt, StructConst, struct_select, struct_error_to_string};
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult, AsyncBlock };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use crate::util::get_instance;
use anyhow::anyhow as err;

simple_interp_command!(GetSwitchInterpCommand,GetSwitchDeserializer,32,4,(0,1,2,3));
simple_interp_command!(ListSwitchInterpCommand,ListSwitchDeserializer,42,4,(0,1,2,3));
simple_interp_command!(SwitchStringInterpCommand,SwitchStringDeserializer,71,3,(0,1,2));
simple_interp_command!(SwitchNumberInterpCommand,SwitchNumberDeserializer,72,3,(0,1,2));
simple_interp_command!(SwitchBooleanInterpCommand,SwitchBooleanDeserializer,73,3,(0,1,2));

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
            out.push(config.get(path));
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Boolean(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ListSwitchInterpCommand {
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
            let path = path_data[*offset..(*offset+*length)].iter().map(|x| x.as_str()).collect::<Vec<_>>();
            if let Some(values) = config.list(&path) {
                out.extend(values.iter().map(|x| x.to_string()));
            }
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}

fn value_to_json(value: &StructBuilt, contents: &[String]) -> Result<Vec<StructConst>,String> {
    Ok(struct_select(value,contents,None)
        .map_err(|e| struct_error_to_string(e))?
        .drain(..).filter_map(|x| x).collect::<Vec<_>>()
    )
}

fn switch_value(r1: &Register, r2: &Register, context: &mut InterpContext) -> anyhow::Result<Vec<StructConst>> {
    let registers = context.registers_mut();
    let switch_data = registers.get_strings(r1)?.to_vec();
    let contents_data = registers.get_strings(r2)?.to_vec();
    drop(registers);
    let request = get_instance::<ShapeRequest>(context,"request")?;
    let config = request.track();
    let path = &switch_data.iter().map(|x| x.as_str()).collect::<Vec<_>>();
    Ok(if let Some(value) = config.value(&path) {
        value_to_json(value,&contents_data).map_err(|e| err!(e))?
    } else {
        vec![]
    })
}

impl InterpCommand for SwitchStringInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let values = switch_value(&self.1,&self.2,context)?;
        let mut out = vec![];
        for value in values {
            let v = match value {
                StructConst::String(s) => s,
                StructConst::Number(n) => n.to_string(),
                StructConst::Boolean(b) => if b { "true" } else { "false" }.to_string(),
                StructConst::Null => "".to_string()
            };
            out.push(v);
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SwitchNumberInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let values = switch_value(&self.1,&self.2,context)?;
        let mut out = vec![];
        for value in values {
            let v = match value {
                StructConst::String(s) => s.parse::<f64>().ok().unwrap_or(0.),
                StructConst::Number(n) => n,
                StructConst::Boolean(b) => if b { 1. } else { 0. },
                StructConst::Null => 0.
            };
            out.push(v);
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Numbers(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SwitchBooleanInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let values = switch_value(&self.1,&self.2,context)?;
        let mut out = vec![];
        for value in values {
            out.push(value.truthy());
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Boolean(out));
        Ok(CommandResult::SyncResult())
    }
}
