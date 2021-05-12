use crate::simple_interp_command;
use peregrine_data::{
    SeaEndPair, SeaEnd, ScreenEdge, ShipEnd, Colour, DirectColour, Patina, ZMenu, Pen, Plotter, DataMessage, Builder,
    ShapeList
};
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use std::cmp::max;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use crate::util::{ get_peregrine, get_instance };

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
simple_interp_command!(ZMenuInterpCommand,ZMenuDeserializer,34,2,(0,1));
simple_interp_command!(PatinaZMenuInterpCommand,PatinaZMenuDeserializer,35,8,(0,1,2,3,4,5,6,7));
simple_interp_command!(UseAllotmentInterpCommand,UseAllotmentDeserializer,41,2,(0,1));

simple_interp_command!(PatinaFilledInterpCommand,PatinaFilledDeserializer,29,2,(0,1));
simple_interp_command!(PatinaHollowInterpCommand,PatinaHollowDeserializer,32,2,(0,1));
simple_interp_command!(DirectColourInterpCommand,DirectColourDeserializer,33,4,(0,1,2,3));
simple_interp_command!(PenInterpCommand,PenDeserializer,36,4,(0,1,2,3));
simple_interp_command!(PlotterInterpCommand,PlotterDeserializer,38,3,(0,1,2));

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

fn patina_colour<F>(context: &mut InterpContext, out: &Register, colour: &Register, cb: F) -> anyhow::Result<()>
        where F: FnOnce(Vec<DirectColour>) -> Patina {
    let registers = context.registers_mut();
    let colour_ids = registers.get_indexes(colour)?.to_vec();
    drop(registers);
    let peregrine = get_peregrine(context)?;
    let geometry_builder = peregrine.geometry_builder();
    let mut map : HashMap<usize,DirectColour> = HashMap::new();
    let mut colours = vec![];
    // XXX too much cloning
    for colour_id in &colour_ids {
        let colour = match map.entry(*colour_id) {
            Entry::Occupied(v) => v.get().clone(),
            Entry::Vacant(v) => {
                let colour = geometry_builder.direct_colour(*colour_id as u32)?;
                v.insert(colour.as_ref().clone());
                colour.as_ref().clone()
            }
        };
        colours.push(colour);
    }
    drop(peregrine);
    let patina = cb(colours);
    let peregrine = get_peregrine(context)?;
    let id = peregrine.geometry_builder().add_patina(patina);
    let registers = context.registers_mut();
    registers.write(out,InterpValue::Indexes(vec![id as usize]));
    Ok(())
}

impl InterpCommand for DirectColourInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let red = registers.get_numbers(&self.1)?.to_vec();
        let green = registers.get_numbers(&self.2)?.to_vec();
        let blue = registers.get_numbers(&self.3)?.to_vec();
        let (red_len,green_len,blue_len) = (red.len(),green.len(),blue.len());
        let len = max(max(red_len,green_len),blue_len);
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();    
        let mut colours = vec![];
        for i in 0..len {
            let dc = DirectColour(red[i%red_len] as u8,green[i%green_len] as u8,blue[i%blue_len] as u8);
            colours.push(geometry_builder.add_direct_colour(dc) as usize);
        }
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(colours));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for UseAllotmentInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let mut name = registers.get_strings(&self.1)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder(); 
        let mut allotment_petitioner = peregrine.allotments().clone();
        let handles = name.drain(..).map(|name| {
            Ok(allotment_petitioner.lookup(&name).ok_or_else(||
                DataMessage::NoSuchAllotment(name)
            )?)
        }).collect::<Result<Vec<_>,DataMessage>>()?;
        let ids = handles.iter().map(|handle| {
            geometry_builder.add_allotment(handle.clone()) as usize           
        }).collect();
        drop(peregrine);
        let zoo = get_instance::<Builder<ShapeList>>(context,"out")?;
        for handle in &handles {
            zoo.lock().add_allotment(handle);
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(ids));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PatinaFilledInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        patina_colour(context,&self.0,&self.1, |dc| Patina::Filled(Colour::Direct(dc)))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PatinaHollowInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        patina_colour(context,&self.0,&self.1, |dc| Patina::Hollow(Colour::Direct(dc)))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ZMenuInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let specs = registers.get_strings(&self.1)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let mut out = vec![];
        for spec in &specs {
            let zmenu = ZMenu::new(spec)?;
            out.push(geometry_builder.add_zmenu(zmenu) as usize);
        }
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(out));
        Ok(CommandResult::SyncResult())
    }
}

fn make_values(keys: &[String], value_d: &[String], value_a: &[usize], value_b: &[usize]) -> anyhow::Result<HashMap<String,Vec<String>>> {
    let mut out = HashMap::new();
    let value_pos = value_a.iter().zip(value_b.iter().cycle());
    let kv = keys.iter().zip(value_pos.cycle());
    for (key,(value_start,value_length)) in kv {
        let values = &value_d[*value_start..(*value_start+*value_length)];
        out.insert(key.to_string(),values.to_vec());
    }
    Ok(out)
}

/* 0: out/patina  1: zmenu  2: key/D  3: key/A  4: key/B  5: value/D  6: value/A  7: value/B */
impl InterpCommand for PatinaZMenuInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let zmenu_ids = registers.get_indexes(&self.1)?;
        let key_d = registers.get_strings(&self.2)?.to_vec();
        let key_a = registers.get_indexes(&self.3)?;
        let key_b = registers.get_indexes(&self.4)?;
        let value_d = registers.get_strings(&self.5)?.to_vec();
        let value_a = registers.get_indexes(&self.6)?.to_vec();
        let value_b = registers.get_indexes(&self.7)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let zmenus : anyhow::Result<Vec<_>> = zmenu_ids.iter().map(|id| geometry_builder.zmenu(*id as u32)).collect();
        let zmenus = zmenus?;
        let key_pos = key_a.iter().zip(key_b.iter().cycle());
        let each = zmenus.iter().zip(key_pos.cycle());
        let mut payload = vec![];
        for (zmenu,(key_start,key_length)) in each {
            let keys = &key_d[*key_start..(*key_start+*key_length)];
            let values = make_values(keys,&value_d,&value_a,&value_b)?;
            let patina = Patina::ZMenu(zmenu.as_ref().clone(),values);
            payload.push(geometry_builder.add_patina(patina) as usize);
        }
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(payload));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PenInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let font = registers.get_strings(&self.1)?[0].to_string();
        let size = registers.get_numbers(&self.2)?[0];
        let colour_ids = registers.get_indexes(&self.3)?;
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let colours : anyhow::Result<Vec<_>> = colour_ids.iter().map(|id| geometry_builder.direct_colour(*id as u32)).collect();
        let colours : Vec<DirectColour> = colours?.iter().map(|x| x.as_ref().clone()).collect();
        let pen = Pen(font,size as u32,colours);
        let id = geometry_builder.add_pen(pen);
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PlotterInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let height = registers.get_numbers(&self.1)?[0];
        let colour_id = registers.get_indexes(&self.2)?[0];
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let colour = geometry_builder.direct_colour(colour_id as u32)?;
        let plotter = Plotter(height,colour.as_ref().clone());
        let id = geometry_builder.add_plotter(plotter);
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));
        Ok(CommandResult::SyncResult())
    }
}
