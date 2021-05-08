use crate::simple_interp_command;
use peregrine_data::{
    AnchorPair, AnchorPairAxis, SingleAnchorAxis, SingleAnchor, Builder, ShapeList
};
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register };
use serde_cbor::Value as CborValue;
use crate::util::{ get_instance, get_peregrine };

simple_interp_command!(Rectangle2InterpCommand,Rectangle2Deserializer,19,8,(0,1,2,3,4,5,6,7));
simple_interp_command!(Rectangle1InterpCommand,Rectangle1Deserializer,20,8,(0,1,2,3,4,5,6,7));
simple_interp_command!(TextInterpCommand,TextDeserializer,37,7,(0,1,2,3,4,5,6));
simple_interp_command!(WiggleInterpCommand,WiggleDeserializer,7,6,(0,1,2,3,4,5));

impl InterpCommand for Rectangle2InterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let sea_x_id = registers.get_indexes(&self.0)?.to_vec();
        let ship_x0_id = registers.get_indexes(&self.1)?.to_vec();
        let ship_x1_id = registers.get_indexes(&self.2)?.to_vec();
        let sea_y_id = registers.get_indexes(&self.3)?.to_vec();
        let ship_y0_id = registers.get_indexes(&self.4)?.to_vec();
        let ship_y1_id = registers.get_indexes(&self.5)?.to_vec();
        let patina_id = registers.get_indexes(&self.6)?.to_vec();
        let allotment_id = registers.get_indexes(&self.7)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let sea_x = geometry.seaendpair(sea_x_id[0] as u32)?.as_ref().clone();
        let ship_x0 = geometry.shipend(ship_x0_id[0] as u32)?.as_ref().clone();
        let ship_x1 = geometry.shipend(ship_x1_id[0] as u32)?.as_ref().clone();
        let sea_y = geometry.seaendpair(sea_y_id[0] as u32)?.as_ref().clone();
        let ship_y0 = geometry.shipend(ship_y0_id[0] as u32)?.as_ref().clone();
        let ship_y1 = geometry.shipend(ship_y1_id[0] as u32)?.as_ref().clone();
        let patina = geometry.patina(patina_id[0] as u32)?.as_ref().clone();
        let mut allotments = vec![];
        for id in &allotment_id {
            allotments.push(geometry.allotment(*id as u32)?.to_string());
        }
        let zoo = get_instance::<Builder<ShapeList>>(context,"out")?;
        zoo.lock().add_rectangle_2(AnchorPair(AnchorPairAxis(sea_x,ship_x0,ship_x1),
                                                   AnchorPairAxis(sea_y,ship_y0,ship_y1)),
                                        patina,allotments);
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for Rectangle1InterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let sea_x_id = registers.get_indexes(&self.0)?.to_vec();
        let ship_x_id = registers.get_indexes(&self.1)?.to_vec();
        let size_x = registers.get_numbers(&self.2)?.to_vec();
        let sea_y_id = registers.get_indexes(&self.3)?.to_vec();
        let ship_y_id = registers.get_indexes(&self.4)?.to_vec();
        let size_y = registers.get_numbers(&self.5)?.to_vec();
        let patina_id = registers.get_indexes(&self.6)?.to_vec();
        let allotment_id = registers.get_indexes(&self.7)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let sea_x = geometry.seaend(sea_x_id[0] as u32)?.as_ref().clone();
        let ship_x = geometry.shipend(ship_x_id[0] as u32)?.as_ref().clone();
        let sea_y = geometry.seaend(sea_y_id[0] as u32)?.as_ref().clone();
        let ship_y = geometry.shipend(ship_y_id[0] as u32)?.as_ref().clone();
        let patina = geometry.patina(patina_id[0] as u32)?.as_ref().clone();
        let mut allotments = vec![];
        for id in &allotment_id {
            allotments.push(geometry.allotment(*id as u32)?.to_string());
        }
        let zoo = get_instance::<Builder<ShapeList>>(context,"out")?;
        zoo.lock().add_rectangle_1(SingleAnchor(SingleAnchorAxis(sea_x,ship_x),
                                                     SingleAnchorAxis(sea_y,ship_y)),
                                        patina, allotments,size_x,size_y);
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for TextInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let sea_x_id = registers.get_indexes(&self.0)?.to_vec();
        let ship_x_id = registers.get_indexes(&self.1)?.to_vec();
        let sea_y_id = registers.get_indexes(&self.2)?.to_vec();
        let ship_y_id = registers.get_indexes(&self.3)?.to_vec();
        let pen_id = registers.get_indexes(&self.4)?.to_vec();
        let text = registers.get_strings(&self.5)?.to_vec();
        let allotment_id = registers.get_indexes(&self.6)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let sea_x = geometry.seaend(sea_x_id[0] as u32)?.as_ref().clone();
        let ship_x = geometry.shipend(ship_x_id[0] as u32)?.as_ref().clone();
        let sea_y = geometry.seaend(sea_y_id[0] as u32)?.as_ref().clone();
        let ship_y = geometry.shipend(ship_y_id[0] as u32)?.as_ref().clone();
        let pen = geometry.pen(pen_id[0] as u32)?.as_ref().clone();
        let mut allotments = vec![];
        for id in &allotment_id {
            allotments.push(geometry.allotment(*id as u32)?.to_string());
        }
        let zoo = get_instance::<Builder<ShapeList>>(context,"out")?;
        zoo.lock().add_text(SingleAnchor(SingleAnchorAxis(sea_x,ship_x),SingleAnchorAxis(sea_y,ship_y)),pen,text,allotments);
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
        let allotment = geometry.allotment(allotment_id as u32)?.to_string();
        let zoo = get_instance::<Builder<ShapeList>>(context,"out")?;
        zoo.lock().add_wiggle(x_min,x_max,plotter,values,allotment);
        Ok(CommandResult::SyncResult())
    }
}
