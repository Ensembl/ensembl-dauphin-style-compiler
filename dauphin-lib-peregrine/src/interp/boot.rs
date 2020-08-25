use anyhow::{ bail, anyhow as err };
use blackbox::blackbox_log;
use peregrine_core::{ Channel, ChannelLocation, InstancePayload };
use dauphin_interp::command::{ InterpLibRegister, CommandDeserializer, InterpCommand };
use dauphin_interp::runtime::{ InterpContext, Register };
use serde_cbor::Value as CborValue;
use crate::PeregrinePayload;

pub struct AddStickAuthorityInterpCommand(Register);

impl InterpCommand for AddStickAuthorityInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<()> {
        let mut self_channel = None;
        if let Some(instance) = context.payload("peregrine","instance")?.as_any_mut().downcast_mut::<InstancePayload>() {
            if let Some(channel) = instance.get("channel") {
                if let Some(channel) = channel.downcast_ref::<Channel>() {
                    self_channel = Some(channel.clone());
                }
            }
        }
        let self_channel = self_channel.ok_or_else(|| err!("could not get self channel"))?;
        let registers = context.registers_mut();
        let authorities = registers.get_strings(&self.0)?;
        if let Some(pc) = context.payload("peregrine","core")?.as_any_mut().downcast_mut::<PeregrinePayload>() {
            for auth in authorities.iter() {
                pc.stick_authority_store().add(&Channel::parse(&self_channel,auth)?);
            }
        }
        Ok(())
    }
}

pub struct AddStickAuthorityDeserializer();

impl CommandDeserializer for AddStickAuthorityDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((0,1))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(AddStickAuthorityInterpCommand(Register::deserialize(&value[0])?)))
    }
}
