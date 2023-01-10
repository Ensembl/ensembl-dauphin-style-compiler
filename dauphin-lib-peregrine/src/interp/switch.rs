use crate::simple_interp_command;
use eachorevery::eoestruct::{StructConst, StructValue};
use peregrine_data::ShapeRequest;
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use crate::util::get_instance;
use anyhow::anyhow as err;

simple_interp_command!(SettingStringInterpCommand,SettingStringDeserializer,0,3,(0,1,2));
simple_interp_command!(SettingNumberInterpCommand,SettingNumberDeserializer,1,3,(0,1,2));
simple_interp_command!(SettingBooleanInterpCommand,SettingBooleanDeserializer,2,3,(0,1,2));
simple_interp_command!(SettingNullInterpCommand,SettingNullDeserializer,3,3,(0,1,2));

fn to_const(value: &StructValue) -> Option<StructConst> {
    match value {
        StructValue::Const(c) => Some(c.clone()),
        _ => None
    }
}

fn value_to_atom(value: &StructValue, contents: &[String]) -> Result<Vec<StructConst>,String> {
    let contents = contents.iter().map(|x| x.as_str()).collect::<Vec<_>>();
    Ok(match value.extract(&contents).ok() {
        Some(StructValue::Const(c)) => vec![c],
        Some(StructValue::Array(a)) => a.iter().filter_map(|x| to_const(x)).collect(),
        Some(StructValue::Object(obj)) => obj.keys().map(|x| StructConst::String(x.clone())).collect(),
        None => vec![]
    })
}

fn setting_value(r1: &Register, r2: &Register, context: &mut InterpContext, is_null_test: bool) -> anyhow::Result<Vec<StructConst>> {
    let registers = context.registers_mut();
    let switch_data = registers.get_strings(r1)?.to_vec();
    let contents_data = registers.get_strings(r2)?.to_vec();
    drop(registers);
    let request = get_instance::<ShapeRequest>(context,"request")?;
    let config = request.track();
    let settings = &switch_data.iter().map(|x| x.as_str()).collect::<Vec<_>>();
    let mut out = vec![];
    //TODO flattens without keeping record, should have advanced option to preserve structure.
    for setting in settings {
        let mut value = if let Some(value) = config.value(&setting) {
            value_to_atom(value,&contents_data).map_err(|e| err!(e))?
        } else if is_null_test && contents_data.len() == 0 {
            vec![StructConst::Null]
        } else {
            vec![]
        };
        out.append(&mut value);
    }
    Ok(out)
}

impl InterpCommand for SettingStringInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let values = setting_value(&self.1,&self.2,context,false)?;
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

impl InterpCommand for SettingNumberInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let values = setting_value(&self.1,&self.2,context,false)?;
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

impl InterpCommand for SettingBooleanInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let values = setting_value(&self.1,&self.2,context,false)?;
        let mut out = vec![];
        for value in values {
            out.push(value.truthy());
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Boolean(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SettingNullInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let values = setting_value(&self.1,&self.2,context,true)?;
        let mut out = vec![];
        for value in values {
            out.push(if let StructConst::Null = value { true } else { false });
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Boolean(out));
        Ok(CommandResult::SyncResult())
    }
}
