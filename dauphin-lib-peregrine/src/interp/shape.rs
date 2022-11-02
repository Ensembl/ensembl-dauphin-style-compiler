use std::sync::{Arc, Mutex};

use anyhow::anyhow as err;
use peregrine_toolkit::lock;
use crate::simple_interp_command;
use peregrine_data::{SpaceBaseArea, PartialSpaceBase, ProgramShapesBuilder, AccessorResolver, DataMessage};
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult, AsyncBlock };
use dauphin_interp::runtime::{ InterpContext, Register };
use serde_cbor::Value as CborValue;
use crate::util::{get_instance, get_peregrine, vec_to_eoe};

simple_interp_command!(Text2InterpCommand,Text2Deserializer,19,4,(0,1,2,3));
simple_interp_command!(WiggleInterpCommand,WiggleDeserializer,7,6,(0,1,2,3,4,5));
simple_interp_command!(RectangleInterpCommand,RectangleDeserializer,20,4,(0,1,2,3));
simple_interp_command!(ImageInterpCommand,ImageDeserializer,44,3,(0,1,2));
simple_interp_command!(EmptyInterpCommand,EmptyDeserializer,53,3,(0,1,2));
simple_interp_command!(RunningTextInterpCommand,RunningTextDeserializer,74,5,(0,1,2,3,4));

impl InterpCommand for RectangleInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let top_left_id = registers.get_indexes(&self.0)?.to_vec();
        let bottom_right_id = registers.get_indexes(&self.1)?.to_vec();
        let patina_id = registers.get_indexes(&self.2)?.to_vec();
        let allotment_id = vec_to_eoe(registers.get_indexes(&self.3)?.to_vec());
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let top_left = geometry.spacebase(top_left_id[0] as u32)?.as_ref().clone();
        if top_left.len() > 0 {
            let bottom_right = geometry.spacebase(bottom_right_id[0] as u32)?.as_ref().clone();
            let patina = geometry.patina(patina_id[0] as u32)?.as_ref().clone();
            let allotments = allotment_id.map_results::<_,_,anyhow::Error>(|id| {
                Ok(geometry.allotment(*id as u32)?.as_ref().clone())
            })?.index(|a| a.name().clone());
            let zoo = get_instance::<Arc<Mutex<Option<ProgramShapesBuilder>>>>(context,"out")?;
            if allotments.len() != Some(0) {
                let area = SpaceBaseArea::new(
                    PartialSpaceBase::from_spacebase(top_left),
                    PartialSpaceBase::from_spacebase(bottom_right)).ok_or_else(|| err!("sb1"))?;
                let area = area.replace_allotments(allotments);
                lock!(zoo).as_mut().unwrap().add_rectangle(area,patina,None)?;
            }
        }
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for Text2InterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let spacebase_id = registers.get_indexes(&self.0)?.to_vec();
        let pen_id = registers.get_indexes(&self.1)?.to_vec();
        let text = vec_to_eoe(registers.get_strings(&self.2)?.to_vec());
        let allotment_id = vec_to_eoe(registers.get_indexes(&self.3)?.to_vec());
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let spacebase = geometry.spacebase(spacebase_id[0] as u32)?.as_ref().clone();
        if spacebase.len() > 0 {
            let pen = geometry.pen(pen_id[0] as u32)?.as_ref().clone();
            let allotments = allotment_id.map_results(|id| {
                geometry.allotment(*id as u32).map(|x| x.as_ref().clone())
            })?.index(|a| a.name().clone());
            let zoo = get_instance::<Arc<Mutex<Option<ProgramShapesBuilder>>>>(context,"out")?;
            if text.len() != Some(0) && allotments.len() != Some(0) {
                let spacebase = spacebase.replace_allotments(allotments);
                lock!(zoo).as_mut().unwrap().add_text(spacebase,pen,text)?;
            }
        }
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for RunningTextInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let topleft_id = registers.get_indexes(&self.0)?.to_vec();
        let bottomright_id = registers.get_indexes(&self.1)?.to_vec();
        let pen_id = registers.get_indexes(&self.2)?.to_vec();
        let text = vec_to_eoe(registers.get_strings(&self.3)?.to_vec());
        let allotment_id = vec_to_eoe(registers.get_indexes(&self.4)?.to_vec());
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let top_left = geometry.spacebase(topleft_id[0] as u32)?.as_ref().clone();
        if top_left.len() > 0 {
            let bottom_right = geometry.spacebase(bottomright_id[0] as u32)?.as_ref().clone();
            let pen = geometry.pen(pen_id[0] as u32)?.as_ref().clone();
            let allotments = allotment_id.map_results(|id| {
                geometry.allotment(*id as u32).map(|x| x.as_ref().clone())
            })?.index(|a| a.name().clone());
            let zoo = get_instance::<Arc<Mutex<Option<ProgramShapesBuilder>>>>(context,"out")?;
            if text.len() != Some(0) || allotments.len() != Some(0) {
                let area = SpaceBaseArea::new(
                    PartialSpaceBase::from_spacebase(top_left),
                    PartialSpaceBase::from_spacebase(bottom_right)).ok_or_else(|| err!("sb5"))?;
                let area = area.replace_allotments(allotments);
                lock!(zoo).as_mut().unwrap().add_running_text(area,pen,text)?;
            }
        }
        Ok(CommandResult::SyncResult())
    }
}

async fn image_command(context: &mut InterpContext, cmd: ImageInterpCommand) -> anyhow::Result<()> {
    let registers = context.registers_mut();
    let spacebase_id = registers.get_indexes(&cmd.0)?.to_vec();
    let images = vec_to_eoe(registers.get_strings(&cmd.1)?.to_vec());
    let allotment_id = vec_to_eoe(registers.get_indexes(&cmd.2)?.to_vec());
    drop(registers);
    let channel_resolver = get_instance::<AccessorResolver>(context,"channel-resolver")?;
    let self_channel = channel_resolver.resolve("self://").await.map_err(|e| DataMessage::XXXTransitional(e))?;
    let peregrine = get_peregrine(context)?;
    let geometry = peregrine.geometry_builder();
    let spacebase = geometry.spacebase(spacebase_id[0] as u32)?.as_ref().clone();
    if spacebase.len() > 0 {
        let allotments = allotment_id.map_results(|id| {
            geometry.allotment(*id as u32).map(|x| x.as_ref().clone())
        })?.index(|a| a.name().clone());
        if images.len() != Some(0) && allotments.len() != Some(0) {
            let spacebase = spacebase.replace_allotments(allotments);
            let zoo = get_instance::<Arc<Mutex<Option<ProgramShapesBuilder>>>>(context,"out")?;
            lock!(zoo).as_mut().unwrap().add_image(&self_channel,spacebase,images)?;
        }
    }
    Ok(())
}

impl InterpCommand for ImageInterpCommand {
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(image_command(context,cmd))))))
    }
}

impl InterpCommand for WiggleInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let x_min = registers.get_numbers(&self.0)?[0];
        let x_max = registers.get_numbers(&self.1)?[0];
        let plotter_id = registers.get_indexes(&self.2)?[0];
        let mut values = registers.get_numbers(&self.3)?.to_vec();
        let present = registers.get_boolean(&self.4)?.to_vec();
        let allotment_id = registers.get_indexes(&self.5)?[0].clone();
        let values = values.drain(..).zip(present.iter().cycle()).map(|(v,p)| if *p { Some(v) } else { None }).collect();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let plotter = geometry.plotter(plotter_id as u32)?.as_ref().clone();
        let allotment = geometry.allotment(allotment_id as u32)?;
        let zoo = get_instance::<Arc<Mutex<Option<ProgramShapesBuilder>>>>(context,"out")?;
        lock!(zoo).as_mut().unwrap().add_wiggle(x_min,x_max,plotter,values,allotment.as_ref().clone())?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for EmptyInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let top_left_id = registers.get_indexes(&self.0)?.to_vec();
        let bottom_right_id = registers.get_indexes(&self.1)?.to_vec();
        let allotment_id = vec_to_eoe(registers.get_indexes(&self.2)?.to_vec());
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let top_left = geometry.spacebase(top_left_id[0] as u32)?.as_ref().clone();
        if top_left.len() > 0 {
            let bottom_right = geometry.spacebase(bottom_right_id[0] as u32)?.as_ref().clone();
            let allotments = allotment_id.map_results::<_,_,anyhow::Error>(|id| {
                Ok(geometry.allotment(*id as u32)?.as_ref().clone())
            })?.index(|a| a.name().clone());
            let zoo = get_instance::<Arc<Mutex<Option<ProgramShapesBuilder>>>>(context,"out")?;
            if allotments.len() != Some(0) {
                let area = SpaceBaseArea::new(
                    PartialSpaceBase::from_spacebase(top_left),
                    PartialSpaceBase::from_spacebase(bottom_right)).ok_or_else(|| err!("sb1"))?;
                let area = area.replace_allotments(allotments);
                lock!(zoo).as_mut().unwrap().add_empty(area)?;
            }
        }
        Ok(CommandResult::SyncResult())
    }
}
