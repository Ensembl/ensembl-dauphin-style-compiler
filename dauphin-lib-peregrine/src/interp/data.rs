use crate::simple_interp_command;
use crate::util::{ get_instance, get_peregrine };
use dauphin_interp::command::{ InterpLibRegister, CommandDeserializer, InterpCommand, AsyncBlock, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue, RegisterFile };
use peregrine_core::{ StickId, issue_stick_request, Stick, StickTopology, Panel, Channel, Scale, Focus, Track, DataStore };
use serde_cbor::Value as CborValue;
use web_sys::console;

simple_interp_command!(GetPanelInterpCommand,GetPanelDeserializer,21,5,(0,1,2,3,4));
simple_interp_command!(GetDataInterpCommand,GetDataDeserializer,22,8,(0,1,2,3,4,5,6,7));

impl InterpCommand for GetPanelInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let panel = get_instance::<Panel>(context,"panel")?;
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Strings(vec![panel.stick_id().get_id().to_string()]));
        registers.write(&self.1,InterpValue::Numbers(vec![panel.index() as f64]));
        registers.write(&self.2,InterpValue::Numbers(vec![panel.scale().get_index() as f64]));
        registers.write(&self.3,InterpValue::Strings(vec![panel.track().name().to_string()]));
        if let Some(focus) = panel.focus().name() {
            registers.write(&self.4,InterpValue::Strings(vec![focus.to_string()]));
        } else {
            registers.write(&self.4,InterpValue::Strings(vec![]));
        }
        Ok(CommandResult::SyncResult())
    }
}

fn get_panel(registers: &RegisterFile, cmd: GetDataInterpCommand) -> anyhow::Result<Option<Panel>> {
    if registers.len(&cmd.3)? == 0 { return Ok(None); }
    let stick = &registers.get_strings(&cmd.3)?[0];
    let index = &registers.get_numbers(&cmd.4)?[0];
    let scale = &registers.get_numbers(&cmd.5)?[0];
    let track = &registers.get_strings(&cmd.6)?[0];
    let focus = &registers.get_strings(&cmd.7)?;
    let focus = focus.get(0);
    Ok(Some(Panel::new(StickId::new(stick),*index as u64,Scale::new(*scale as u64),Focus::new(focus.map(|x| &x as &str)),Track::new(track))))
}

async fn get(context: &mut InterpContext, cmd: GetDataInterpCommand) -> anyhow::Result<()> {
    let self_channel = get_instance::<Channel>(context,"channel")?;
    let registers = context.registers_mut();
    let channel_name = registers.get_strings(&cmd.1)?;
    let prog_name = &registers.get_strings(&cmd.2)?[0];
    if let Some(panel) = get_panel(registers,cmd)? {
        let peregrine = get_peregrine(context)?;
        let data_store = peregrine.data_store();    
        let channel = Channel::parse(&self_channel,&channel_name[0])?;
        let result = data_store.get(&panel,&channel,prog_name).await?;
    }
    Ok(())
}

impl InterpCommand for GetDataInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(get(context,cmd))))))
    }
}
