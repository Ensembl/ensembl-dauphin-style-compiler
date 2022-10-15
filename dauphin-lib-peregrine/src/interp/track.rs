use crate::simple_interp_command;
use peregrine_data::{ AccessorResolver, DataMessage };
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult, AsyncBlock };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use peregrine_toolkit::eachorevery::eoestruct::StructTemplate;
use peregrine_toolkit::lock;
use serde_cbor::Value as CborValue;
use crate::util::{ get_instance, get_peregrine };

simple_interp_command!(NewLaneInterpCommand,NewLaneDeserializer,4,6,(0,1,2,3,4,5));
simple_interp_command!(AddTagInterpCommand,AddTagDeserializer,5,2,(0,1));
simple_interp_command!(AddTriggerInterpCommand,AddTriggerDeserializer,6,4,(0,1,2,3));
simple_interp_command!(AddSwitchInterpCommand,AddSwitchDeserializer,11,4,(0,1,2,3));
simple_interp_command!(DataSourceInterpCommand,DataSourceDeserializer,8,1,(0));
simple_interp_command!(SetSwitchInterpCommand,SetSwitchDeserializer,33,4,(0,1,2,3));
simple_interp_command!(ClearSwitchInterpCommand,ClearSwitchDeserializer,34,4,(0,1,2,3));
simple_interp_command!(AppendGroupInterpCommand,AppendGroupDeserializer,47,3,(0,1,2));
simple_interp_command!(AppendDepthInterpCommand,AppendDepthDeserializer,48,3,(0,1,2));

async fn new_lane_interp_command(context: &mut InterpContext, cmd: NewLaneInterpCommand) -> anyhow::Result<()> {
    let channel_resolver = get_instance::<AccessorResolver>(context,"channel-resolver")?;
    let registers = context.registers_mut();
    let channels = registers.get_strings(&cmd.1)?.to_vec();
    let programs = registers.get_strings(&cmd.2)?.to_vec();
    let min_scale = registers.get_indexes(&cmd.3)?.to_vec();
    let max_scale = registers.get_indexes(&cmd.4)?.to_vec();
    let scale_jump = registers.get_indexes(&cmd.5)?.to_vec();
    drop(registers);
    let mut track_ids = vec![];
    let values = channels.iter().cycle().zip(programs.iter());
    let scales = min_scale.iter().cycle().zip(max_scale.iter().cycle());
    let mut scales = scales.zip(scale_jump.iter().cycle());
    let track_builder = get_peregrine(context)?.track_builder();
    for (channel_name,program) in values {
        let channel = channel_resolver.resolve(&channel_name).await.map_err(|e| DataMessage::XXXTransitional(e))?;
        let ((min,max),jump) = scales.next().unwrap();
        track_ids.push(track_builder.allocate(&channel,program,*min as u64,*max as u64,*jump as u64));
    }
    let registers = context.registers_mut();
    registers.write(&cmd.0,InterpValue::Indexes(track_ids));
    Ok(())
}

impl InterpCommand for NewLaneInterpCommand {
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(new_lane_interp_command(context,cmd))))))
    }
}

impl InterpCommand for AddTagInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let track_ids = registers.get_indexes(&self.0)?.to_vec();
        let tags = registers.get_strings(&self.1)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        for track_id in &track_ids {
            let track = peregrine.track_builder().get(*track_id)?;
            let mut track = track.lock().unwrap();
            for tag in &tags {
                track.add_tag(tag);
            }
            drop(track);
        }
        Ok(CommandResult::SyncResult())
    }
}

fn add_mount(track_ids: &[usize], track_d: &[String], track_a: &[usize], track_b: &[usize], context: &mut InterpContext, trigger: bool) -> anyhow::Result<()> {
    let peregrine = get_peregrine(context)?;
    let track_pos = track_a.iter().cycle().zip(track_b.iter().cycle());
    let data = track_ids.iter().zip(track_pos);
    for (track_id,(track_a,track_b)) in data {
        let track = peregrine.track_builder().get(*track_id)?;
        let path = &track_d[*track_a..(*track_a+*track_b)];
        let path : Vec<_> = path.iter().map(|x| x.as_str()).collect();
        lock!(track).add_mount(&path,trigger);
    }
    Ok(())
}

fn track_switch(track_ids: &[usize], track_d: &[String], track_a: &[usize], track_b: &[usize], context: &mut InterpContext, yn: bool) -> anyhow::Result<()> {
    let peregrine = get_peregrine(context)?;
    let track_pos = track_a.iter().cycle().zip(track_b.iter().cycle());
    let data = track_ids.iter().zip(track_pos);
    for (track_id,(track_a,track_b)) in data {
        let track = peregrine.track_builder().get(*track_id)?;
        let path = &track_d[*track_a..(*track_a+*track_b)];
        let path : Vec<_> = path.iter().map(|x| x.as_str()).collect();
        let value = StructTemplate::new_boolean(yn).build().unwrap();
        lock!(track).set_switch(&path,value);
    }
    Ok(())
}

impl InterpCommand for AddTriggerInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let track_ids = registers.get_indexes(&self.0)?.to_vec();
        let track_d = registers.get_strings(&self.1)?.to_vec();
        let track_a = registers.get_indexes(&self.2)?.to_vec();
        let track_b = registers.get_indexes(&self.3)?.to_vec();
        drop(registers);
        add_mount(&track_ids,&track_d,&track_a,&track_b,context,true)?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for AddSwitchInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let track_ids = registers.get_indexes(&self.0)?.to_vec();
        let track_d = registers.get_strings(&self.1)?.to_vec();
        let track_a = registers.get_indexes(&self.2)?.to_vec();
        let track_b = registers.get_indexes(&self.3)?.to_vec();
        drop(registers);
        add_mount(&track_ids,&track_d,&track_a,&track_b,context,false)?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SetSwitchInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let track_ids = registers.get_indexes(&self.0)?.to_vec();
        let track_d = registers.get_strings(&self.1)?.to_vec();
        let track_a = registers.get_indexes(&self.2)?.to_vec();
        let track_b = registers.get_indexes(&self.3)?.to_vec();
        drop(registers);
        track_switch(&track_ids,&track_d,&track_a,&track_b,context,true)?;
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ClearSwitchInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let track_ids = registers.get_indexes(&self.0)?.to_vec();
        let track_d = registers.get_strings(&self.1)?.to_vec();
        let track_a = registers.get_indexes(&self.2)?.to_vec();
        let track_b = registers.get_indexes(&self.3)?.to_vec();
        drop(registers);
        track_switch(&track_ids,&track_d,&track_a,&track_b,context,false)?;
        Ok(CommandResult::SyncResult())
    }
}

async fn data_source(context: &mut InterpContext, cmd: DataSourceInterpCommand) -> anyhow::Result<()> {
    let registers = context.registers_mut();
    let track_ids = registers.get_indexes(&cmd.0)?.to_vec();
    drop(registers);
    let peregrine = get_peregrine(context)?;
    let track_builder = peregrine.track_builder().clone();
    for track_id in &track_ids {
        let track_builder = track_builder.get(*track_id)?;
        let mut track_builder = track_builder.lock().unwrap();
        track_builder.build(peregrine.switches());
    }
    Ok(())
}

impl InterpCommand for DataSourceInterpCommand {
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(data_source(context,cmd))))))
    }
}

impl InterpCommand for AppendGroupInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let allotments = registers.get_strings(&self.1)?.to_vec();
        let groups = registers.get_strings(&self.2)?.to_vec();
        let mut out = vec![];
        for (allotment,group) in allotments.iter().zip(groups.iter().cycle()) {
            if allotment.is_empty() {
                out.push("".to_string());
            } else {
                out.push(format!("{}/{}",allotment,group));
            }
        }
        registers.write(&self.0, InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for AppendDepthInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let allotments = registers.get_strings(&self.1)?.to_vec();
        let depths = registers.get_numbers(&self.2)?.to_vec();
        let mut out = vec![];
        for (allotment,depth) in allotments.iter().zip(depths.iter().cycle()) {
            if allotment.is_empty() {
                out.push("".to_string());
            } else {
                out.push(format!("{}[{}]",allotment,depth));
            }
        }
        registers.write(&self.0, InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}
