use crate::simple_interp_command;
use crate::util::{ get_instance, get_peregrine };
use dauphin_interp::command::{ InterpLibRegister, CommandDeserializer, InterpCommand, AsyncBlock, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use peregrine_core::{ StickId, issue_stick_request, Stick, StickTopology, Panel };
use serde_cbor::Value as CborValue;

simple_interp_command!(GetPanelInterpCommand,GetPanelDeserializer,21,5,(0,1,2,3,4));

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
