use std::sync::Mutex;

use crate::simple_interp_command;
use crate::util::{get_peregrine, vec_to_eoe};
use dauphin_interp::runtime::{ Register, InterpContext, InterpValue };
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
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
        let number = vec_to_eoe(registers.get_numbers(&self.2)?.to_vec());
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let group = geometry_builder.eoegroup(group_id)?;
        let id = geometry_builder.add_eoevar(StructVar::new_number(&mut *lock!(group),number));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesVarStringInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let group_id = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.) as u32;
        let strings = vec_to_eoe(registers.get_strings(&self.2)?.to_vec());
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let group = geometry_builder.eoegroup(group_id)?;
        let id = geometry_builder.add_eoevar(StructVar::new_string(&mut *lock!(group),strings));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesVarBooleanInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let group_id = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.) as u32;
        let booleans = vec_to_eoe(registers.get_boolean(&self.2)?.to_vec());
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let group = geometry_builder.eoegroup(group_id)?;
        let id = geometry_builder.add_eoevar(StructVar::new_boolean(&mut *lock!(group),booleans));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesNullInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_null());
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesArrayInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let inner_ids = vec_to_eoe(registers.get_numbers(&self.1)?.iter().map(|x| *x as u32).collect::<Vec<_>>());
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let inners = inner_ids.map_results(|id| geometry_builder.eoetmpl(*id).map(|x| x.as_ref().clone()))?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_array(inners));
        drop(peregrine);
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
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let value = geometry_builder.eoetmpl(value_id)?;
        let id = geometry_builder.add_eoepair(StructPair::new(&key,value.as_ref().clone()));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesObjectInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let inner_ids = vec_to_eoe(registers.get_numbers(&self.1)?.iter().map(|x| *x as u32).collect::<Vec<_>>());
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let inners = inner_ids.map_results(|id| geometry_builder.eoepair(*id).map(|x| x.as_ref().clone()))?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_object(inners));
        drop(peregrine);
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
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let var = geometry_builder.eoevar(var_id)?;
        let expr = geometry_builder.eoetmpl(expr_id)?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_condition(var.as_ref().clone(),expr.as_ref().clone()));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesGroupInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let eoe_group = Mutex::new(StructVarGroup::new());
        let id = geometry_builder.add_eoegroup(eoe_group);
        drop(peregrine);
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
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let group = geometry_builder.eoegroup(group_id)?;
        let expr = geometry_builder.eoetmpl(expr_id)?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_all(&mut *lock!(group),expr.as_ref().clone()));
        drop(peregrine);
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
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let var = geometry_builder.eoevar(var_id)?;
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_var(&var));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesNumberInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let number = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.);
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_number(number));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesStringInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let string = registers.get_strings(&self.1)?.get(0).cloned().unwrap_or(String::new());
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_string(string));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesBooleanInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let boolean = registers.get_boolean(&self.1)?.get(0).cloned().unwrap_or(false);
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let id = geometry_builder.add_eoetmpl(StructTemplate::new_boolean(boolean));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EoesLateInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let group_id = registers.get_numbers(&self.1)?.get(0).cloned().unwrap_or(0.) as u32;
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let group = geometry_builder.eoegroup(group_id)?;
        let late = StructTemplate::new_var(&StructVar::new_late(&mut *lock!(group)));
        let id = geometry_builder.add_eoetmpl(late);
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));
        Ok(CommandResult::SyncResult())
    }
}