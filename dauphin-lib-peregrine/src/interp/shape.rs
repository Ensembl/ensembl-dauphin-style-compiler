use crate::simple_interp_command;
use anyhow::{ bail, anyhow as err };
use peregrine_core::{
    Track, Scale, Channel, SeaEndPair, SeaEnd, ScreenEdge, ShipEnd, PanelRunOutput, AnchorPair, Patina, Colour, AnchorPairAxis, DirectColour,
    SingleAnchorAxis, SingleAnchor
};
use dauphin_interp::command::{ InterpLibRegister, CommandDeserializer, InterpCommand, AsyncBlock, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue, RegisterFile };
use serde_cbor::Value as CborValue;
use crate::util::{ get_instance, get_peregrine };
use web_sys::console;

simple_interp_command!(Rectangle2InterpCommand,Rectangle2Deserializer,19,6,(0,1,2,3,4,5));
simple_interp_command!(Rectangle1InterpCommand,Rectangle1Deserializer,20,6,(0,1,2,3,4,5));

impl InterpCommand for Rectangle2InterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let sea_x_id = registers.get_indexes(&self.0)?.to_vec();
        let ship_x0_id = registers.get_indexes(&self.1)?.to_vec();
        let ship_x1_id = registers.get_indexes(&self.2)?.to_vec();
        let sea_y_id = registers.get_indexes(&self.3)?.to_vec();
        let ship_y0_id = registers.get_indexes(&self.4)?.to_vec();
        let ship_y1_id = registers.get_indexes(&self.5)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let sea_x = geometry.seaendpair(sea_x_id[0] as u32)?.as_ref().clone();
        let ship_x0 = geometry.shipend(ship_x0_id[0] as u32)?.as_ref().clone();
        let ship_x1 = geometry.shipend(ship_x1_id[0] as u32)?.as_ref().clone();
        let sea_y = geometry.seaendpair(sea_y_id[0] as u32)?.as_ref().clone();
        let ship_y0 = geometry.shipend(ship_y0_id[0] as u32)?.as_ref().clone();
        let ship_y1 = geometry.shipend(ship_y1_id[0] as u32)?.as_ref().clone();
        let out = get_instance::<PanelRunOutput>(context,"out")?;
        let zoo = out.zoo();
        zoo.rectangle().add_rectangle_2(AnchorPair(AnchorPairAxis(sea_x,ship_x0,ship_x1),
                                                   AnchorPairAxis(sea_y,ship_y0,ship_y1)),
                                        Patina::Filled(Colour::Direct(vec![DirectColour(0,0,0)])));
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
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let geometry = peregrine.geometry_builder();
        let sea_x = geometry.seaend(sea_x_id[0] as u32)?.as_ref().clone();
        let ship_x = geometry.shipend(ship_x_id[0] as u32)?.as_ref().clone();
        let sea_y = geometry.seaend(sea_y_id[0] as u32)?.as_ref().clone();
        let ship_y = geometry.shipend(ship_y_id[0] as u32)?.as_ref().clone();
        let out = get_instance::<PanelRunOutput>(context,"out")?;
        let zoo = out.zoo();
        zoo.rectangle().add_rectangle_1(SingleAnchor(SingleAnchorAxis(sea_x,ship_x,size_x),
                                                     SingleAnchorAxis(sea_y,ship_y,size_y)),
                                        Patina::Filled(Colour::Direct(vec![DirectColour(0,0,0)])));
        Ok(CommandResult::SyncResult())
    }
}
