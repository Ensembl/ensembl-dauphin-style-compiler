use crate::simple_interp_command;
use anyhow::{ bail, anyhow as err };
use peregrine_core::{ Track, Scale, Channel, SeaEndPair, SeaEnd, ScreenEdge, ShipEnd };
use dauphin_interp::command::{ InterpLibRegister, CommandDeserializer, InterpCommand, AsyncBlock, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use crate::util::{ get_instance, get_peregrine };

simple_interp_command!(IntervalInterpCommand,IntervalDeserializer,9,3,(0,1,2));
simple_interp_command!(ScreenStartPairInterpCommand,ScreenStartPairDeserializer,10,3,(0,1,2));
simple_interp_command!(ScreenEndPairInterpCommand,ScreenEndPairDeserializer,11,3,(0,1,2));
simple_interp_command!(ScreenSpanPairInterpCommand,ScreenSpanPairDeserializer,12,3,(0,1,2));

simple_interp_command!(PositionInterpCommand,PositionDeserializer,13,2,(0,1));
simple_interp_command!(ScreenStartInterpCommand,ScreenStartDeserializer,14,2,(0,1));
simple_interp_command!(ScreenEndInterpCommand,ScreenEndDeserializer,15,2,(0,1));

simple_interp_command!(PinStartInterpCommand,PinStartDeserializer,16,2,(0,1));
simple_interp_command!(PinCentreInterpCommand,PinCentreDeserializer,17,2,(0,1));
simple_interp_command!(PinEndInterpCommand,PinEndDeserializer,18,2,(0,1));

fn seaendpair<F>(context: &mut InterpContext, out: &Register, starts: &Register, ends: &Register, cb: F) -> anyhow::Result<()>
                where F: FnOnce(Vec<f64>,Vec<f64>) -> SeaEndPair {
    let registers = context.registers_mut();
    let starts = registers.get_numbers(&starts)?.to_vec();
    let ends = registers.get_numbers(&ends)?.to_vec();
    drop(registers);
    let peregrine = get_peregrine(context)?;
    let id = peregrine.geometry_builder().add_seaendpair(cb(starts,ends));
    let registers = context.registers_mut();
    registers.write(&out,InterpValue::Indexes(vec![id as usize]));
    Ok(())
}

fn seaend<F>(context: &mut InterpContext, out: &Register, pos: &Register, cb: F) -> anyhow::Result<()>
                where F: FnOnce(Vec<f64>) -> SeaEnd {
    let registers = context.registers_mut();
    let pos = registers.get_numbers(&pos)?.to_vec();
    drop(registers);
    let peregrine = get_peregrine(context)?;
    let id = peregrine.geometry_builder().add_seaend(cb(pos));
    let registers = context.registers_mut();
    registers.write(&out,InterpValue::Indexes(vec![id as usize]));
    Ok(())
}

fn shipend<F>(context: &mut InterpContext, out: &Register, pos: &Register, cb: F) -> anyhow::Result<()>
                where F: FnOnce(Vec<f64>) -> ShipEnd {
    let registers = context.registers_mut();
    let pos = registers.get_numbers(&pos)?.to_vec();
    drop(registers);
    let peregrine = get_peregrine(context)?;
    let id = peregrine.geometry_builder().add_shipend(cb(pos));
    let registers = context.registers_mut();
    registers.write(&out,InterpValue::Indexes(vec![id as usize]));
    Ok(())
}

impl InterpCommand for IntervalInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        seaendpair(context,&self.0,&self.1,&self.2,|starts,ends| SeaEndPair::Paper(starts,ends))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ScreenStartPairInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        seaendpair(context,&self.0,&self.1,&self.2,|starts,ends| SeaEndPair::Screen(ScreenEdge::Min(starts),ScreenEdge::Min(ends)))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ScreenEndPairInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        seaendpair(context,&self.0,&self.1,&self.2,|starts,ends| SeaEndPair::Screen(ScreenEdge::Max(starts),ScreenEdge::Max(ends)))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ScreenSpanPairInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        seaendpair(context,&self.0,&self.1,&self.2,|starts,ends| SeaEndPair::Screen(ScreenEdge::Min(starts),ScreenEdge::Max(ends)))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PositionInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        seaend(context,&self.0,&self.1,|pos| SeaEnd::Paper(pos))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ScreenStartInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        seaend(context,&self.0,&self.1,|pos| SeaEnd::Screen(ScreenEdge::Min(pos)))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ScreenEndInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        seaend(context,&self.0,&self.1,|pos| SeaEnd::Screen(ScreenEdge::Max(pos)))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PinStartInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        shipend(context,&self.0,&self.1,|pos| ShipEnd::Min(pos))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PinCentreInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        shipend(context,&self.0,&self.1,|pos| ShipEnd::Centre(pos))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PinEndInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        shipend(context,&self.0,&self.1,|pos| ShipEnd::Max(pos))?;
        Ok(CommandResult::SyncResult())
    }
}
