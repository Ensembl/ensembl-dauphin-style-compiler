use crate::simple_interp_command;
use peregrine_data::{ ProgramName, Scale, Channel, Track };
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult, AsyncBlock };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use crate::util::{ get_instance, get_peregrine };

simple_interp_command!(NewLaneInterpCommand,NewLaneDeserializer,4,3,(0,1,2));
simple_interp_command!(AddTagInterpCommand,AddTagDeserializer,5,2,(0,1));
simple_interp_command!(AddTriggerInterpCommand,AddTriggerDeserializer,6,4,(0,1,2,3));
simple_interp_command!(AddSwitchInterpCommand,AddSwitchDeserializer,6,4,(0,1,2,3));
simple_interp_command!(SetScaleInterpCommand,SetScaleDeserializer,7,3,(0,1,2));
simple_interp_command!(DataSourceInterpCommand,DataSourceDeserializer,8,1,(0));
simple_interp_command!(LaneSetMaxScaleJumpInterpCommand,LaneSetMaxScaleJumpDeserializer,40,2,(0,1));

impl InterpCommand for NewLaneInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let self_channel = get_instance::<Channel>(context,"channel")?;
        let registers = context.registers_mut();
        let channels = registers.get_strings(&self.1)?.to_vec();
        let programs = registers.get_strings(&self.2)?.to_vec();
        drop(registers);
        let mut lane_ids = vec![];
        let values = channels.iter().cycle().zip(programs.iter());
        let lane_builder = get_peregrine(context)?.lane_builder();
        for (channel,program) in values {
            let channel = Channel::parse(&self_channel,channel)?;
            lane_ids.push(lane_builder.allocate(&channel,program));
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(lane_ids));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for AddTagInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let lane_ids = registers.get_indexes(&self.0)?.to_vec();
        let tags = registers.get_strings(&self.1)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        for lane_id in &lane_ids {
            let lane = peregrine.lane_builder().get(*lane_id)?;
            let mut lane = lane.lock().unwrap();
            let old_tags = lane.prb.program_region_mut().stick_tags().map(|x| x.to_vec()).unwrap_or(vec![]);
            let mut new_tags = old_tags.to_vec();
            new_tags.extend(tags.iter().cloned());
            lane.prb.program_region_mut().set_stick_tags(&new_tags);
            drop(lane);
        }
        Ok(CommandResult::SyncResult())
    }
}

fn add_mount(lane_ids: &[usize], track_d: &[String], track_a: &[usize], track_b: &[usize], context: &mut InterpContext, trigger: bool) -> anyhow::Result<()> {
    let peregrine = get_peregrine(context)?;
    let track_pos = track_a.iter().cycle().zip(track_b.iter().cycle());
    let data = lane_ids.iter().zip(track_pos);
    for (lane_id,(track_a,track_b)) in data {
        let lane = peregrine.lane_builder().get(*lane_id)?;
        let path = &track_d[*track_a..(*track_a+*track_b)];
        let path : Vec<_> = path.iter().map(|x| x.as_str()).collect();
        lane.lock().unwrap().prb.add_mount(&path,trigger);
    }
    Ok(())
}

impl InterpCommand for AddTriggerInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let lane_ids = registers.get_indexes(&self.0)?.to_vec();
        let track_d = registers.get_strings(&self.1)?.to_vec();
        let track_a = registers.get_indexes(&self.2)?.to_vec();
        let track_b = registers.get_indexes(&self.3)?.to_vec();
        drop(registers);
        add_mount(&lane_ids,&track_d,&track_a,&track_b,context,true)?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for AddSwitchInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let lane_ids = registers.get_indexes(&self.0)?.to_vec();
        let track_d = registers.get_strings(&self.1)?.to_vec();
        let track_a = registers.get_indexes(&self.2)?.to_vec();
        let track_b = registers.get_indexes(&self.3)?.to_vec();
        drop(registers);
        add_mount(&lane_ids,&track_d,&track_a,&track_b,context,false)?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SetScaleInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let lane_ids = registers.get_indexes(&self.0)?.to_vec();
        let scale_from = registers.get_indexes(&self.1)?.to_vec();
        let scale_to = registers.get_indexes(&self.2)?.to_vec();
        let scale : Vec<(_,_)> = scale_from.iter().zip(scale_to.iter().cycle()).map(|(x,y)| (*x,*y)).collect();
        let mut scale_iter = scale.iter().cycle();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        for lane_id in &lane_ids {
            let (from,to) = scale_iter.next().unwrap();
            let lane = peregrine.lane_builder().get(*lane_id)?;
            let mut lane = lane.lock().unwrap();
            lane.prb.program_region_mut().set_scale(Scale::new(*from as u64),Scale::new(*to as u64));
            drop(lane);
        }
        Ok(CommandResult::SyncResult())
    }
}

async fn data_source(context: &mut InterpContext, cmd: DataSourceInterpCommand) -> anyhow::Result<()> {
    let registers = context.registers_mut();
    let lane_ids = registers.get_indexes(&cmd.0)?.to_vec();
    drop(registers);
    let peregrine = get_peregrine(context)?;
    let lane_program_lookup = peregrine.agent_store().lane_program_lookup().await.clone();
    let lane_builder = peregrine.lane_builder().clone();
    for lane_id in &lane_ids {
        let track_builder = lane_builder.get(*lane_id)?;
        let track = track_builder.lock().unwrap().track.clone();
        let program_region = track_builder.lock().unwrap().prb.build(&track,peregrine.switches());
        lane_program_lookup.add(&program_region,track.program_name());
    }
    Ok(())
}

impl InterpCommand for DataSourceInterpCommand {
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(data_source(context,cmd))))))
    }
}

impl InterpCommand for LaneSetMaxScaleJumpInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let lane_ids = registers.get_indexes(&self.0)?.to_vec();
        let max_jump = registers.get_indexes(&self.1)?.to_vec();
        let mut max_jump_iter = max_jump.iter().cycle();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        for lane_id in &lane_ids {
            let max_jump = max_jump_iter.next().unwrap();
            let lane = peregrine.lane_builder().get(*lane_id)?;
            let mut lane = lane.lock().unwrap();
            lane.prb.program_region_mut().set_max_scale_jump(*max_jump as u32);
            drop(lane);
        }
        Ok(CommandResult::SyncResult())
    }
}
