use crate::simple_interp_command;
use peregrine_core::{ Track, Scale, Channel };
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use crate::util::{ get_instance, get_peregrine };

simple_interp_command!(NewPanelInterpCommand,NewPanelDeserializer,4,1,(0));
simple_interp_command!(AddTagInterpCommand,AddTagDeserializer,5,2,(0,1));
simple_interp_command!(AddTrackInterpCommand,AddTrackDeserializer,6,2,(0,1));
simple_interp_command!(SetScaleInterpCommand,SetScaleDeserializer,7,3,(0,1,2));
simple_interp_command!(DataSourceInterpCommand,DataSourceDeserializer,8,3,(0,1,2));
simple_interp_command!(PanelSetMaxScaleJumpInterpCommand,PanelSetMaxScaleJumpDeserializer,40,2,(0,1));

impl InterpCommand for NewPanelInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let panel_id = get_peregrine(context)?.panel_builder().allocate();
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![panel_id]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for AddTagInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let panel_ids = registers.get_indexes(&self.0)?.to_vec();
        let tags = registers.get_strings(&self.1)?.to_vec();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        for panel_id in &panel_ids {
            let panel = peregrine.panel_builder().get(*panel_id)?;
            let mut panel = panel.lock().unwrap();
            let old_tags = panel.stick_tags().map(|x| x.to_vec()).unwrap_or(vec![]);
            let mut new_tags = old_tags.to_vec();
            new_tags.extend(tags.iter().cloned());
            panel.set_stick_tags(&new_tags);
            drop(panel);
        }
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for AddTrackInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let panel_ids = registers.get_indexes(&self.0)?.to_vec();
        let tracks : Vec<_> = registers.get_strings(&self.1)?.iter().map(|x| Track::new(x)).collect();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        for panel_id in &panel_ids {
            let panel = peregrine.panel_builder().get(*panel_id)?;
            let mut panel = panel.lock().unwrap();
            let old_tracks = panel.tracks().map(|x| x.to_vec()).unwrap_or(vec![]);
            let mut new_tracks = old_tracks.to_vec();
            new_tracks.extend(tracks.iter().cloned());
            panel.set_tracks(&new_tracks);
            drop(panel);
        }
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SetScaleInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let panel_ids = registers.get_indexes(&self.0)?.to_vec();
        let scale_from = registers.get_indexes(&self.1)?.to_vec();
        let scale_to = registers.get_indexes(&self.2)?.to_vec();
        let scale : Vec<(_,_)> = scale_from.iter().zip(scale_to.iter().cycle()).map(|(x,y)| (*x,*y)).collect();
        let mut scale_iter = scale.iter().cycle();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        for panel_id in &panel_ids {
            let (from,to) = scale_iter.next().unwrap();
            let panel = peregrine.panel_builder().get(*panel_id)?;
            let mut panel = panel.lock().unwrap();
            panel.set_scale(Scale::new(*from as u64),Scale::new(*to as u64));
            drop(panel);
        }
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for DataSourceInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let self_channel = get_instance::<Channel>(context,"channel")?;
        let registers = context.registers_mut();
        let panel_ids = registers.get_indexes(&self.0)?.to_vec();
        let channels = registers.get_strings(&self.1)?.to_vec();
        let prog_names = registers.get_strings(&self.2)?.to_vec();
        let programs : Vec<(_,_)> = prog_names.iter().zip(channels.iter().cycle()).map(|(x,y)| (y.to_string(),x.to_string())).collect();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let panel_program_store = peregrine.panel_program_store().clone();
        let panel_builder = peregrine.panel_builder().clone();
        let mut programs = programs.iter().cycle();
        for panel_id in &panel_ids {
            let (channel,name) = programs.next().unwrap();
            let channel = Channel::parse(&self_channel,channel)?;
            let panel = panel_builder.get(*panel_id)?;
            panel_program_store.add(&panel.lock().unwrap(),&channel,name);
        }
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for PanelSetMaxScaleJumpInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let panel_ids = registers.get_indexes(&self.0)?.to_vec();
        let max_jump = registers.get_indexes(&self.1)?.to_vec();
        let mut max_jump_iter = max_jump.iter().cycle();
        drop(registers);
        let peregrine = get_peregrine(context)?;
        for panel_id in &panel_ids {
            let max_jump = max_jump_iter.next().unwrap();
            let panel = peregrine.panel_builder().get(*panel_id)?;
            let mut panel = panel.lock().unwrap();
            panel.set_max_scale_jump(*max_jump as u32);
            drop(panel);
        }
        Ok(CommandResult::SyncResult())
    }
}
