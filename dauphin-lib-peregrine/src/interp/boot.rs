use crate::simple_interp_command;
use crate::util::{ get_instance, get_peregrine };
use peregrine_data::{ Channel };
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, AsyncBlock, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use peregrine_data::{ StickId, issue_stick_request, Stick, StickTopology };
use serde_cbor::Value as CborValue;
use crate::payloads::PeregrinePayload;

simple_interp_command!(AddStickAuthorityInterpCommand,AddStickAuthorityDeserializer,0,1,(0));
simple_interp_command!(GetStickIdInterpCommand,GetStickIdDeserializer,1,1,(0));
simple_interp_command!(GetStickDataInterpCommand,GetStickDataDeserializer,2,8,(0,1,2,3,4,5,6,7));
simple_interp_command!(AddStickInterpCommand,AddStickDeserializer,3,6,(0,1,2,3,4,5));

// TODO booted is a mess,  is it needed?
async fn add_stick_authority(context: &mut InterpContext, cmd: AddStickAuthorityInterpCommand) -> anyhow::Result<()> {
    let self_channel = get_instance::<Channel>(context,"channel")?;
    let registers = context.registers_mut();
    let authorities = registers.get_strings(&cmd.0)?;
    if let Some(pc) = context.payload("peregrine","core")?.as_any_mut().downcast_mut::<PeregrinePayload>() {
        pc.booted().lock();
        let agent_store = pc.agent_store().clone();
        let stick_authority_store = agent_store.stick_authority_store().await.clone();
        let mut tasks = vec![];
        for auth in authorities.iter() {
            let task = stick_authority_store.add(Channel::parse(&self_channel,auth)?);
            tasks.push(task);
        }
        for task in tasks {
            task.await?;
        }
        pc.booted().unlock();
    }
    Ok(())
}

impl InterpCommand for AddStickAuthorityInterpCommand {
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(add_stick_authority(context,cmd))))))
    }
}

impl InterpCommand for GetStickIdInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let my_stick_id = get_instance::<StickId>(context,"stick_id")?;
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Strings(vec![my_stick_id.get_id().to_string()]));
        Ok(CommandResult::SyncResult())
    }
}

async fn get(context: &mut InterpContext, cmd: GetStickDataInterpCommand) -> anyhow::Result<()> {
    let self_channel = get_instance::<Channel>(context,"channel")?;
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
    let pc = get_peregrine(context)?;
    for stick_id in id_strings.iter() {
        let channel_name = channel_iter.next().unwrap();
        let stick = issue_stick_request(pc.manager().clone(),Channel::parse(&self_channel,channel_name)?,StickId::new(stick_id)).await?;
        id_strings_out.push(stick.get_id().get_id().to_string());
        sizes.push(stick.size() as f64);
        topologies.push(stick.topology().to_number() as usize);
        let mut tags: Vec<_> = stick.tags().iter().cloned().collect();
        tags_offsets.push(tags_data.len());
        tags_lengths.push(tags.len());
        tags_data.append(&mut tags);
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
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(get(context,cmd))))))
    }
}

async fn add_stick(context: &mut InterpContext, cmd: AddStickInterpCommand) -> anyhow::Result<()> {
    let registers = context.registers_mut();
    let sizes = registers.get_numbers(&cmd.1)?;
    let topologies = registers.get_numbers(&cmd.2)?;
    let tags_data = registers.get_strings(&cmd.3)?;
    let tags_offsets = registers.get_indexes(&cmd.4)?;
    let tags_lengths = registers.get_indexes(&cmd.5)?;
    let mut sizes = sizes.iter().cycle();
    let mut topologies = topologies.iter().cycle();
    let mut tags_offsets = tags_offsets.iter().cycle();
    let mut tags_lengths = tags_lengths.iter().cycle();
    let ids = registers.get_strings(&cmd.0)?;
    let mut sticks = vec![];
    for id in ids.iter() {
        let size = sizes.next().unwrap();
        let topology = topologies.next().unwrap();
        let tags_offset = tags_offsets.next().unwrap();
        let tags_length = tags_lengths.next().unwrap();
        let tags = tags_data[(*tags_offset..(tags_offset+tags_length))].to_vec();
        sticks.push(Stick::new(&StickId::new(id),*size as u64,StickTopology::from_number(*topology as u64 as u8)?,&tags));
    }
    let pc = get_peregrine(context)?;
    let stick_store = pc.agent_store().stick_store().await;
    for stick in sticks.drain(..) {
        stick_store.add(stick.get_id().clone(),stick);
    }
    Ok(())
}

impl InterpCommand for AddStickInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(add_stick(context,cmd))))))
    }
}
