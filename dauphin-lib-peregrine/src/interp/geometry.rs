use anyhow::anyhow as err;
use peregrine_toolkit::eachorevery::EachOrEvery;
use peregrine_toolkit::{lock};
use crate::simple_interp_command;
use peregrine_data::{Colour, DirectColour, DrawnType, Patina, Pen, Plotter, ShapeRequest, ZMenu, SpaceBase, ProgramShapesBuilder, Hotspot, Background, AttachmentPoint};
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use std::cmp::max;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::util::{get_instance, get_peregrine, vec_to_eoe};

simple_interp_command!(ZMenuInterpCommand,ZMenuDeserializer,14,2,(0,1));
simple_interp_command!(PatinaZMenuInterpCommand,PatinaZMenuDeserializer,15,8,(0,1,2,3,4,5,6,7));
simple_interp_command!(UseAllotmentInterpCommand,UseAllotmentDeserializer,12,2,(0,1));
simple_interp_command!(PatinaFilledInterpCommand,PatinaFilledDeserializer,29,2,(0,1));
simple_interp_command!(PatinaHollowInterpCommand,PatinaHollowDeserializer,9,3,(0,1,2));
simple_interp_command!(DirectColourInterpCommand,DirectColourDeserializer,13,5,(0,1,2,3,4));
simple_interp_command!(PenInterpCommand,PenDeserializer,16,5,(0,1,2,3,4));
simple_interp_command!(PlotterInterpCommand,PlotterDeserializer,18,3,(0,1,2));
simple_interp_command!(SpaceBaseInterpCommand,SpaceBaseDeserializer,17,4,(0,1,2,3));
simple_interp_command!(SimpleColourInterpCommand,SimpleColourDeserializer,35,2,(0,1));
simple_interp_command!(StripedInterpCommand,StripedDeserializer,36,6,(0,1,2,3,4,5));
simple_interp_command!(BarredInterpCommand,BarredDeserializer,37,6,(0,1,2,3,4,5));
simple_interp_command!(BpRangeInterpCommand,BpRangeDeserializer,45,1,(0));
simple_interp_command!(SpotColourInterpCommand,SpotColourDeserializer,46,2,(0,1));
simple_interp_command!(PpcInterpCommand,PpcDeserializer,49,1,(0));
simple_interp_command!(StyleInterpCommand,StyleDeserializer,50,3,(0,1,2));
simple_interp_command!(PatinaSwitchInterpCommand,PatinaSwitchDeserializer,51,3,(0,1,2));
simple_interp_command!(PatinaMetadataInterpCommand,PatinaMetadataDeserializer,54,4,(0,1,2,3));
simple_interp_command!(BackgroundInterpCommand,BackgroundDeserializer,70,3,(0,1,2));

impl InterpCommand for BpRangeInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let shape = get_instance::<ShapeRequest>(context,"request")?;
        let region = shape.region();
        let registers = context.registers_mut();
        let min = region.min_value();
        let max = region.max_value();
        registers.write(&self.0,InterpValue::Numbers(vec![min as f64, max as f64]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PpcInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let shape = get_instance::<ShapeRequest>(context,"request")?;
        let pixel_size = shape.pixel_size();
        let registers = context.registers_mut();
        let min = pixel_size.min_px_per_carriage();
        let max = pixel_size.max_px_per_carriage();
        registers.write(&self.0,InterpValue::Numbers(vec![min as f64, max as f64]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SpaceBaseInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let mut base = vec_to_eoe(registers.get_numbers(&self.1)?.to_vec());
        let normal = vec_to_eoe(registers.get_numbers(&self.2)?.to_vec());
        let tangent = vec_to_eoe(registers.get_numbers(&self.3)?.to_vec());
        if base.len().is_none() && normal.len().is_none() && tangent.len().is_none() {
            base = base.to_each(1).unwrap();
        }
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let spacebase = SpaceBase::new(&base,&normal,&tangent,&EachOrEvery::every(())).ok_or_else(|| err!("sb4"))?;
        let id = geometry_builder.add_spacebase(spacebase);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![id as usize]));    
        Ok(CommandResult::SyncResult())
    }
}

fn patina_colour<F>(context: &mut InterpContext, out: &Register, colour: &Register, cb: F) -> anyhow::Result<()>
        where F: FnOnce(EachOrEvery<Colour>) -> Patina {
    let registers = context.registers_mut();
    let colour_ids = registers.get_indexes(colour)?.to_vec();
    drop(registers);
    let peregrine = get_peregrine(context)?;
    let geometry_builder = peregrine.geometry_builder();
    let mut colours = vec![];
    for colour_id in &colour_ids {
        colours.push(geometry_builder.colour(*colour_id as u32)?.as_ref().clone());
    }
    drop(peregrine);
    let patina = cb(vec_to_eoe(colours));
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
        let alpha = registers.get_numbers(&self.4)?.to_vec();
        let (red_len,green_len,blue_len,alpha_len) = (red.len(),green.len(),blue.len(),alpha.len());
        let len = max(max(red_len,green_len),max(blue_len,alpha_len));
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();    
        let mut colours = vec![];
        for i in 0..len {
            let dc = DirectColour(red[i%red_len] as u8,green[i%green_len] as u8,blue[i%blue_len] as u8,alpha[i%alpha_len] as u8);
            colours.push(geometry_builder.add_direct_colour(dc) as usize);
        }
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(colours));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SimpleColourInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let direct_ids = registers.get_indexes(&self.1)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();    
        let direct_colour = if let Some(direct_id) = direct_ids.get(0) {
            let dc = geometry_builder.direct_colour(*direct_id as u32)?;
            dc.as_ref().clone()
        } else {
            DirectColour(255,255,255,0)
        };
        let colour_id = geometry_builder.add_colour(Colour::Direct(direct_colour));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![colour_id as usize]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SpotColourInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let direct_ids = registers.get_indexes(&self.1)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();    
        let direct_colour = if let Some(direct_id) = direct_ids.get(0) {
            let dc = geometry_builder.direct_colour(*direct_id as u32)?;
            dc.as_ref().clone()
        } else {
            DirectColour(255,255,255,0)
        };
        let colour_id = geometry_builder.add_colour(Colour::Spot(direct_colour));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![colour_id as usize]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for StripedInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let direct_ids_a = registers.get_indexes(&self.1)?.to_vec();
        let direct_ids_b = registers.get_indexes(&self.2)?.to_vec();
        let stripe_x = *registers.get_numbers(&self.3)?.to_vec().get(0).unwrap_or(&2.);
        let stripe_y = *registers.get_numbers(&self.4)?.to_vec().get(0).unwrap_or(&2.);
        let prop = *registers.get_numbers(&self.5)?.to_vec().get(0).unwrap_or(&0.5);
        let stripes = (stripe_x as u32,stripe_y as u32);
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();    
        let direct_colour_a = if let Some(direct_id) = direct_ids_a.get(0) {
            let dc = geometry_builder.direct_colour(*direct_id as u32)?;
            dc.as_ref().clone()
        } else {
            DirectColour(255,255,255,0)
        };
        let direct_colour_b = if let Some(direct_id) = direct_ids_b.get(0) {
            let dc = geometry_builder.direct_colour(*direct_id as u32)?;
            dc.as_ref().clone()
        } else {
            DirectColour(255,255,255,0)
        };
        let colour_id = geometry_builder.add_colour(Colour::Stripe(direct_colour_a,direct_colour_b,stripes,prop));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![colour_id as usize]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for BarredInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let direct_ids_a = registers.get_indexes(&self.1)?.to_vec();
        let direct_ids_b = registers.get_indexes(&self.2)?.to_vec();
        let stripe_x = *registers.get_numbers(&self.3)?.to_vec().get(0).unwrap_or(&2.);
        let stripe_y = *registers.get_numbers(&self.4)?.to_vec().get(0).unwrap_or(&2.);
        let prop = *registers.get_numbers(&self.5)?.to_vec().get(0).unwrap_or(&0.5);
        let stripes = (stripe_x as u32,stripe_y as u32);
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();    
        let direct_colour_a = if let Some(direct_id) = direct_ids_a.get(0) {
            let dc = geometry_builder.direct_colour(*direct_id as u32)?;
            dc.as_ref().clone()
        } else {
            DirectColour(255,255,255,0)
        };
        let direct_colour_b = if let Some(direct_id) = direct_ids_b.get(0) {
            let dc = geometry_builder.direct_colour(*direct_id as u32)?;
            dc.as_ref().clone()
        } else {
            DirectColour(255,255,255,0)
        };
        let colour_id = geometry_builder.add_colour(Colour::Bar(direct_colour_a,direct_colour_b,stripes,prop));
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![colour_id as usize]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for UseAllotmentInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let mut name = registers.get_strings(&self.1)?.to_vec();
        drop(registers);
        let zoo = get_instance::<Arc<Mutex<Option<ProgramShapesBuilder>>>>(context,"out")?;
        let mut shapes_lock = lock!(zoo);
        let shapes = shapes_lock.as_mut().unwrap();
        let requests = name.drain(..).map(|name| shapes.use_allotment(&name).clone()).collect::<Vec<_>>();
        drop(shapes);
        drop(shapes_lock);
        drop(zoo);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder(); 
        let ids = requests.iter().map(|request| {
            geometry_builder.add_allotment(request.clone()) as usize
        }).collect();
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(ids));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PatinaFilledInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        patina_colour(context,&self.0,&self.1, |c| Patina::Drawn(DrawnType::Fill,c))?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PatinaHollowInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let width = *registers.get_numbers(&self.2)?.to_vec().get(0).unwrap_or(&1.);
        drop(registers);    
        patina_colour(context,&self.0,&self.1, |c| Patina::Drawn(DrawnType::Stroke(width),c))?;
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

fn make_values(keys: &[String], value_d: &[String], value_a: &[usize], value_b: &[usize]) -> anyhow::Result<Vec<(String,EachOrEvery<String>)>> {
    let mut out = vec![];
    let value_pos = value_a.iter().zip(value_b.iter().cycle());
    let kv = keys.iter().zip(value_pos.cycle());
    for (key,(value_start,value_length)) in kv {
        let values = &value_d[*value_start..(*value_start+*value_length)];
        out.push((key.to_string(),vec_to_eoe(values.to_vec())));
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
            let patina = Patina::Hotspot(Hotspot::ZMenu(zmenu.as_ref().clone(),values));
            payload.push(geometry_builder.add_patina(patina) as usize);
        }
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(payload));
        Ok(CommandResult::SyncResult())
    }
}

fn make_switches(key: &[String], sense: &[bool]) -> anyhow::Result<Vec<(Vec<String>,bool)>> {
    let mut out = vec![];
    for (key,sense) in key.iter().zip(sense.iter().cycle()) {
        out.push((key.split("/").map(|x| x.to_string()).collect(),*sense));
    }
    Ok(out)
}

/* 0: out/patina  1: key  2: bool */
impl InterpCommand for PatinaSwitchInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let key = registers.get_strings(&self.1)?.to_vec();
        let sense = registers.get_boolean(&self.2)?;
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let values = vec_to_eoe(make_switches(&key,&sense)?);
        let patina = Patina::Hotspot(Hotspot::Switch(values));
        let patina_id = geometry_builder.add_patina(patina) as usize;
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![
            patina_id
        ]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PenInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let font = registers.get_strings(&self.1)?[0].to_string();
        let size = registers.get_numbers(&self.2)?[0];
        let (size,attachment) = if size < 0. {
            (-size,AttachmentPoint::Right)
        } else {
            (size,AttachmentPoint::Left)
        };
        let colour_ids = registers.get_indexes(&self.3)?;
        let background_id = registers.get_indexes(&self.4)?.get(0).cloned();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let colours : anyhow::Result<Vec<_>> = colour_ids.iter().map(|id| geometry_builder.direct_colour(*id as u32)).collect();
        let colours : Vec<DirectColour> = colours?.iter().map(|x| x.as_ref().clone()).collect();
        let background = background_id.map(|id| geometry_builder.background(id as u32)).transpose()?.map(|x| x.as_ref().clone());
        let pen = Pen::new(&font,size as u32,&colours,&background,&attachment);
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

impl InterpCommand for StyleInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let spec = registers.get_strings(&self.0)?[0].clone();        
        let keys = registers.get_strings(&self.1)?;
        let values = registers.get_strings(&self.2)?;
        drop(registers);
        let mut props = HashMap::new();
        for (key,value) in keys.iter().zip(values.iter()) {
            props.insert(key.to_string(),value.to_string());
        }
        let zoo = get_instance::<Arc<Mutex<Option<ProgramShapesBuilder>>>>(context,"out")?;
        lock!(zoo).as_mut().unwrap().add_style(&spec,props);
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PatinaMetadataInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let key = registers.get_strings(&self.1)?[0].to_string();
        let ids = registers.get_strings(&self.2)?.to_vec();
        let value_ids = registers.get_numbers(&self.3)?;
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let values = value_ids.iter().zip(ids.iter()).map(|(request,id)| {
            let tmpl = geometry_builder.eoetmpl(*request as u32)
                .map(|x| x.as_ref().clone());
            Ok::<_,anyhow::Error>((id.clone(),tmpl?))
        }).collect::<Result<Vec<_>,_>>()?;
        let values = EachOrEvery::each(values);
        let patina = Patina::Metadata(key,values);
        let payload = geometry_builder.add_patina(patina) as usize;
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![payload]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for BackgroundInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let colours = registers.get_indexes(&self.1)?;
        let rounded = registers.get_boolean(&self.2)?[0];
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry_builder = peregrine.geometry_builder();
        let colours : anyhow::Result<Vec<_>> = colours.iter().map(|id| geometry_builder.direct_colour(*id as u32)).collect();
        let colours : Vec<DirectColour> = colours?.iter().map(|x| x.as_ref().clone()).collect();
        let ids = colours.iter().map(|colour| {
            let bgd = Background {
                colour: colour.clone(),
                round: rounded
            };
            geometry_builder.add_background(bgd) as usize
        }).collect::<Vec<_>>();
        drop(peregrine);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(ids));
        Ok(CommandResult::SyncResult())
    }
}
