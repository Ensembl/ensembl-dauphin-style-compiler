use std::sync::Mutex;

use crate::simple_interp_command;
use crate::util::{get_instance};
use dauphin_interp::runtime::{ Register, InterpContext, InterpValue, RegisterFile };
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
use peregrine_data::ObjectBuilder;
use peregrine_toolkit::eachorevery::EachOrEvery;
use peregrine_toolkit::eachorevery::eoestruct::{StructVarGroup, StructTemplate, StructVar, StructPair};
use peregrine_toolkit::lock;
use serde_cbor::Value as CborValue;

simple_interp_command!(EoesVarNumberInterpCommand,EoesVarNumberDeserializer,55,3,(0,1,2));
simple_interp_command!(EoesVarStringInterpCommand,EoesVarStringDeserializer,56,3,(0,1,2));
simple_interp_command!(EoesVarBooleanInterpCommand,EoesVarBooleanDeserializer,57,3,(0,1,2));
simple_interp_command!(EoesNullInterpCommand,EoesNullDeserializer,58,1,(0));
simple_interp_command!(EoesArrayInterpCommand,EoesArrayDeserializer,59,2,(0,1));
simple_interp_command!(EoesPairInterpCommand,EoesPairDeserializer,60,3,(0,1,2));
simple_interp_command!(EoesObjectInterpCommand,EoesObjectDeserializer,61,2,(0,1));
simple_interp_command!(EoesConditionInterpCommand,EoesConditionDeserializer,62,3,(0,1,2));
simple_interp_command!(EoesGroupInterpCommand,EoesGroupDeserializer,63,1,(0));
simple_interp_command!(EoesAllInterpCommand,EoesAllDeserializer,64,3,(0,1,2));
simple_interp_command!(EoesVarInterpCommand,EoesVarDeserializer,65,2,(0,1));
simple_interp_command!(EoesNumberInterpCommand,EoesNumberDeserializer,66,2,(0,1));
simple_interp_command!(EoesStringInterpCommand,EoesStringDeserializer,67,2,(0,1));
simple_interp_command!(EoesBooleanInterpCommand,EoesBooleanDeserializer,68,2,(0,1));
simple_interp_command!(EoesLateInterpCommand,EoesLateDeserializer,69,2,(0,1));

impl InterpCommand for EoesVarNumberInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let group_id = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.) as u32;
        let number = EachOrEvery::each(registers.get_numbers(&self.2)?.to_vec());
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let group = geometry_builder.eoegroup(group_id)?;
        let id = geometry_builder.add_eoevar(StructVar::new_number(&mut *lock!(group),number));
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesVarStringInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let group_id = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.) as u32;
        let strings = EachOrEvery::each(registers.get_strings(&self.2)?.to_vec());
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let group = geometry_builder.eoegroup(group_id)?;
        let id = geometry_builder.add_eoevar(StructVar::new_string(&mut *lock!(group),strings));
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesVarBooleanInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let group_id = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.) as u32;
        let booleans = EachOrEvery::each(registers.get_boolean(&self.2)?.to_vec());
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let group = geometry_builder.eoegroup(group_id)?;
        let id = geometry_builder.add_eoevar(StructVar::new_boolean(&mut *lock!(group),booleans));
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesNullInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_null());
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesArrayInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let inner_ids = EachOrEvery::each(registers.get_numbers(&self.1)?.iter().map(|x| *x as u32).collect::<Vec<_>>());
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let inners = inner_ids.map_results(|id| geometry_builder.eoetmpl(*id).map(|x| x.as_ref().clone()))?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_array(inners));
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesPairInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let key = registers.get_strings(&self.1)?.get(0).cloned().unwrap_or("".to_string());
        let value_id = registers.get_numbers(&self.2)?[0] as u32;
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let value = geometry_builder.eoetmpl(value_id)?;
        let id = geometry_builder.add_eoepair(StructPair::new(&key,value.as_ref().clone()));
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesObjectInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let inner_ids = EachOrEvery::each(registers.get_numbers(&self.1)?.iter().map(|x| *x as u32).collect::<Vec<_>>());
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let inners = inner_ids.map_results(|id| geometry_builder.eoepair(*id).map(|x| x.as_ref().clone()))?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_object(inners));
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesConditionInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let var_id = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.) as u32;
        let expr_id = registers.get_numbers(&self.2)?.get(0).cloned().unwrap_or(0.) as u32;
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let var = geometry_builder.eoevar(var_id)?;
        let expr = geometry_builder.eoetmpl(expr_id)?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_condition(var.as_ref().clone(),expr.as_ref().clone()));
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesGroupInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let eoe_group = Mutex::new(StructVarGroup::new());
        let id = geometry_builder.add_eoegroup(eoe_group);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesAllInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let group_id = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.) as u32;
        let expr_id = registers.get_numbers(&self.2)?.get(0).cloned().unwrap_or(0.) as u32;
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let group = geometry_builder.eoegroup(group_id)?;
        let expr = geometry_builder.eoetmpl(expr_id)?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_all(&mut *lock!(group),expr.as_ref().clone()));
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesVarInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let var_id = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.) as u32;
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let var = geometry_builder.eoevar(var_id)?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_var(&var));
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

fn eoes_builder_command<'a,F,G,X>(reg0: &Register, reg1: &Register, context: &mut InterpContext,
                        get: F, build: G) -> anyhow::Result<CommandResult<'a>>
            where F: Fn(&RegisterFile,&Register) -> anyhow::Result<Vec<X>>,
                  G: Fn(X) -> StructTemplate {
    let registers = context.registers();
    let mut values = get(registers,reg1)?;
    drop(registers);
    let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
    let ids = values.drain(..).map(|x| 
        geometry_builder.add_eoetmpl(build(x)) as usize
    ).collect::<Vec<_>>();
    let registers = context.registers_mut();
    registers.write(&reg0,InterpValue::Indexes(ids));    
    Ok(CommandResult::SyncResult())
}

impl InterpCommand for EoesNumberInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        eoes_builder_command(&self.0,&self.1,context,|registers,reg| {
            Ok(registers.get_numbers(&reg)?.to_vec())
        }, |value| StructTemplate::new_number(value))
    }
}

impl InterpCommand for EoesStringInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        eoes_builder_command(&self.0,&self.1,context,|registers,reg| {
            Ok(registers.get_strings(&reg)?.to_vec())
        }, |value| StructTemplate::new_string(value))
    }
}

impl InterpCommand for EoesBooleanInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        eoes_builder_command(&self.0,&self.1,context,|registers,reg| {
            Ok(registers.get_boolean(&reg)?.to_vec())
        }, |value| StructTemplate::new_boolean(value))
    }
}

impl InterpCommand for EoesLateInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let group_id = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.) as u32;
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let group = geometry_builder.eoegroup(group_id)?;
        let late = StructTemplate::new_var(&StructVar::new_late(&mut *lock!(group)));
        let id = geometry_builder.add_eoetmpl(late);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));
        Ok(CommandResult::SyncResult())
    }
}