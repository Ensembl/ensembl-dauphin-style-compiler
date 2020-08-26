use anyhow::{ bail, anyhow as err };
use blackbox::blackbox_log;
use peregrine_core::{ Channel, ChannelLocation, InstancePayload };
use dauphin_interp::command::{ InterpLibRegister, CommandDeserializer, InterpCommand, AsyncBlock, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use peregrine_core::{ StickId, get_stick };
use serde_cbor::Value as CborValue;
use std::pin::Pin;
use std::future::Future;
use crate::PeregrinePayload;

pub struct AddStickAuthorityInterpCommand(Register);

impl InterpCommand for AddStickAuthorityInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
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
                pc.stick_authority_store().add(&Channel::parse(&self_channel,auth)?)?;
            }
        }
        Ok(CommandResult::SyncResult())
    }
}

pub struct GetStickIdInterpCommand(Register);

impl InterpCommand for GetStickIdInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let mut my_stick_id = None;
        if let Some(instance) = context.payload("peregrine","instance")?.as_any_mut().downcast_mut::<InstancePayload>() {
            if let Some(stick_id) = instance.get("stick_id") {
                if let Some(stick_id) = stick_id.downcast_ref::<StickId>() {
                    my_stick_id = Some(stick_id.clone());
                }
            }
        }
        let my_stick_id = my_stick_id.ok_or_else(|| err!("could not get stick id"))?;
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Strings(vec![my_stick_id.get_id().to_string()]));
        Ok(CommandResult::SyncResult())
    }
}

#[derive(Clone)]
pub struct GetStickDataInterpCommand(Register,Register,Register,Register,Register,Register,Register,Register);

async fn get(context: &mut InterpContext, cmd: GetStickDataInterpCommand) -> anyhow::Result<()> {
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
    let channel_iter = registers.get_strings(&cmd.6)?;
    let mut channel_iter = channel_iter.iter().cycle();
    let id_strings = registers.get_strings(&cmd.7)?;
    let mut id_strings_out = vec![];
    let mut sizes = vec![];
    let mut topologies = vec![];
    let mut tags_offsets = vec![];
    let mut tags_lengths = vec![];
    let mut tags_data = vec![];
    drop(registers);
    if let Some(pc) = context.payload("peregrine","core")?.as_any_mut().downcast_mut::<PeregrinePayload>() {
        for stick_id in id_strings.iter() {
            let channel_name = channel_iter.next().unwrap();
            let stick = get_stick(pc.manager().clone(),Channel::parse(&self_channel,channel_name)?,StickId::new(stick_id)).await?;
            id_strings_out.push(stick.get_id().get_id().to_string());
            sizes.push(stick.size() as f64);
            topologies.push(stick.topology().to_number() as usize);
            let mut tags: Vec<_> = stick.tags().iter().cloned().collect();
            tags_offsets.push(tags_data.len());
            tags_lengths.push(tags.len());
            tags_data.append(&mut tags);
        }
    }
    let registers = context.registers_mut();
    registers.write(&cmd.0,InterpValue::Strings(id_strings_out));
    registers.write(&cmd.1,InterpValue::Numbers(sizes));
    registers.write(&cmd.2,InterpValue::Indexes(topologies));
    registers.write(&cmd.3,InterpValue::Strings(tags_data));
    registers.write(&cmd.4,InterpValue::Indexes(tags_offsets));
    registers.write(&cmd.5,InterpValue::Indexes(tags_lengths));
    Ok(())
}

impl InterpCommand for GetStickDataInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(get(context,cmd))))))
    }
}

pub struct AddStickAuthorityDeserializer();

impl CommandDeserializer for AddStickAuthorityDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((0,1))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(AddStickAuthorityInterpCommand(Register::deserialize(&value[0])?)))
    }
}

pub struct GetStickIdDeserializer();

impl CommandDeserializer for GetStickIdDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((1,1))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(GetStickIdInterpCommand(Register::deserialize(&value[0])?)))
    }
}

pub struct GetStickDataDeserializer();

impl CommandDeserializer for GetStickDataDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((2,8))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(GetStickDataInterpCommand(
            Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,Register::deserialize(&value[2])?,Register::deserialize(&value[3])?,
            Register::deserialize(&value[4])?,Register::deserialize(&value[5])?,Register::deserialize(&value[6])?,Register::deserialize(&value[7])?)))
    }
}
