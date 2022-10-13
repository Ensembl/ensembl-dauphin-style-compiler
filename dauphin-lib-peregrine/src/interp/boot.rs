use crate::simple_interp_command;
use crate::util::{ get_instance };
use peregrine_data::{ AccessorResolver, DataMessage };
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, AsyncBlock, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register };
use serde_cbor::Value as CborValue;
use crate::payloads::PeregrinePayload;

simple_interp_command!(AddAuthorityInterpCommand,AddAuthorityDeserializer,0,1,(0));

// TODO booted is a mess,  is it needed?
async fn add_stick_authority(context: &mut InterpContext, cmd: AddAuthorityInterpCommand) -> anyhow::Result<()> {
    let channel_resolver = get_instance::<AccessorResolver>(context,"channel-resolver")?;
    let registers = context.registers_mut();
    let authorities = registers.get_strings(&cmd.0)?;
    if let Some(pc) = context.payload("peregrine","core")?.as_any_mut().downcast_mut::<PeregrinePayload>() {
        pc.booted().lock();
        let agent_store = pc.agent_store().clone();
        let stick_authority_store = agent_store.stick_authority_store.clone();
        let mut tasks = vec![];
        for auth in authorities.iter() {
            let channel = channel_resolver.resolve(&auth).await.map_err(|e| DataMessage::XXXTransitional(e))?;
            let task = stick_authority_store.add(channel);
            tasks.push(task);
        }
        for task in tasks {
            task.await.map_err(|e| DataMessage::XXXTransitional(e))?;
        }
        pc.booted().unlock();
    }
    Ok(())
}

impl InterpCommand for AddAuthorityInterpCommand {
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(add_stick_authority(context,cmd))))))
    }
}
