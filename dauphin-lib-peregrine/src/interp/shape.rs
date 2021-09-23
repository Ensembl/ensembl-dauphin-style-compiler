use crate::simple_interp_command;
use peregrine_data::{Builder, HoleySpaceBase, HoleySpaceBaseArea, ShapeListBuilder, SpaceBaseArea};
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register };
use serde_cbor::Value as CborValue;
use crate::util::{ get_instance, get_peregrine };

simple_interp_command!(Text2InterpCommand,Text2Deserializer,19,4,(0,1,2,3));
simple_interp_command!(WiggleInterpCommand,WiggleDeserializer,7,6,(0,1,2,3,4,5));
simple_interp_command!(RectangleInterpCommand,RectangleDeserializer,20,4,(0,1,2,3));
simple_interp_command!(ImageInterpCommand,ImageDeserializer,44,4,(0,1,2,3));

impl InterpCommand for RectangleInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let top_left_id = registers.get_indexes(&self.0)?.to_vec();
        let bottom_right_id = registers.get_indexes(&self.1)?.to_vec();
        let patina_id = registers.get_indexes(&self.2)?.to_vec();
        let allotment_id = registers.get_indexes(&self.3)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let top_left = geometry.spacebase(top_left_id[0] as u32)?.as_ref().clone();
        let bottom_right = geometry.spacebase(bottom_right_id[0] as u32)?.as_ref().clone();
        let patina = geometry.patina(patina_id[0] as u32)?.as_ref().clone();
        let allotments = allotment_id.iter().map(|id| {
            Ok(geometry.allotment(*id as u32)?.as_ref().clone())
        }).collect::<anyhow::Result<Vec<_>>>()?;
        let zoo = get_instance::<Builder<ShapeListBuilder>>(context,"out")?;
        let area = SpaceBaseArea::new(top_left,bottom_right);
        zoo.lock().add_rectangle(HoleySpaceBaseArea::Simple(area),patina,allotments);
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for Text2InterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let spacebase_id = registers.get_indexes(&self.0)?.to_vec();
        let pen_id = registers.get_indexes(&self.1)?.to_vec();
        let text = registers.get_strings(&self.2)?.to_vec();
        let allotment_id = registers.get_indexes(&self.3)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let spacebase = geometry.spacebase(spacebase_id[0] as u32)?.as_ref().clone();
        let pen = geometry.pen(pen_id[0] as u32)?.as_ref().clone();
        let mut allotments = vec![];
        for id in &allotment_id {
            allotments.push(geometry.allotment(*id as u32)?.as_ref().clone());
        }
        let zoo = get_instance::<Builder<ShapeListBuilder>>(context,"out")?;
        zoo.lock().add_text(HoleySpaceBase::Simple(spacebase),pen,text,allotments);
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ImageInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let spacebase_id = registers.get_indexes(&self.0)?.to_vec();
        let images = registers.get_strings(&self.1)?.to_vec();
        let depth = registers.get_numbers(&self.2)?.to_vec()[0] as i8;
        let allotment_id = registers.get_indexes(&self.3)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let spacebase = geometry.spacebase(spacebase_id[0] as u32)?.as_ref().clone();
        let mut allotments = vec![];
        for id in &allotment_id {
            allotments.push(geometry.allotment(*id as u32)?.as_ref().clone());
        }
        let zoo = get_instance::<Builder<ShapeListBuilder>>(context,"out")?;
        zoo.lock().add_image(HoleySpaceBase::Simple(spacebase),depth,images,allotments);
        Ok(CommandResult::SyncResult())
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
        let zoo = get_instance::<Builder<ShapeListBuilder>>(context,"out")?;
        zoo.lock().add_wiggle(x_min,x_max,plotter,values,allotment.as_ref().clone());
        Ok(CommandResult::SyncResult())
    }
}
