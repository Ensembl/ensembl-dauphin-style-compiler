use crate::simple_interp_command;
use peregrine_data::Channel;
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult, AsyncBlock };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use crate::util::{ get_instance, get_peregrine };

simple_interp_command!(NewLaneInterpCommand,NewLaneDeserializer,4,6,(0,1,2,3,4,5));
simple_interp_command!(AddTagInterpCommand,AddTagDeserializer,5,2,(0,1));
simple_interp_command!(AddTriggerInterpCommand,AddTriggerDeserializer,6,4,(0,1,2,3));
simple_interp_command!(AddSwitchInterpCommand,AddSwitchDeserializer,6,4,(0,1,2,3));
simple_interp_command!(DataSourceInterpCommand,DataSourceDeserializer,8,1,(0));

impl InterpCommand for NewLaneInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let self_channel = get_instance::<Channel>(context,"channel")?;
        let registers = context.registers_mut();
        let channels = registers.get_strings(&self.1)?.to_vec();
        let programs = registers.get_strings(&self.2)?.to_vec();
        let min_scale = registers.get_indexes(&self.3)?.to_vec();
        let max_scale = registers.get_indexes(&self.4)?.to_vec();
        let scale_jump = registers.get_indexes(&self.5)?.to_vec();
        drop(registers);
        let mut lane_ids = vec![];
        let values = channels.iter().cycle().zip(programs.iter());
        let scales = min_scale.iter().cycle().zip(max_scale.iter().cycle());
        let mut scales = scales.zip(scale_jump.iter().cycle());
        let lane_builder = get_peregrine(context)?.lane_builder();
        for (channel,program) in values {
            let channel = Channel::parse(&self_channel,channel)?;
            let ((min,max),jump) = scales.next().unwrap();
            lane_ids.push(lane_builder.allocate(&channel,program,*min as u64,*max as u64,*jump as u64));
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
            for tag in &tags {
                lane.add_tag(tag);
            }
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
        lane.lock().unwrap().add_mount(&path,trigger);
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

async fn data_source(context: &mut InterpContext, cmd: DataSourceInterpCommand) -> anyhow::Result<()> {
    let registers = context.registers_mut();
    let lane_ids = registers.get_indexes(&cmd.0)?.to_vec();
    drop(registers);
    let peregrine = get_peregrine(context)?;
    let lane_program_lookup = peregrine.agent_store().lane_program_lookup().await.clone();
    let lane_builder = peregrine.lane_builder().clone();
    for lane_id in &lane_ids {
        let track_builder = lane_builder.get(*lane_id)?;
        let mut track_builder = track_builder.lock().unwrap();
        let track = track_builder.track().clone();
        let program_region = track_builder.build(peregrine.switches());
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
