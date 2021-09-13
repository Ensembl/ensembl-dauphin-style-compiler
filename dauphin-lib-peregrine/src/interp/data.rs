use std::sync::Mutex;
use std::sync::Arc;
use crate::simple_interp_command;
use crate::util::{ get_instance, get_peregrine };
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, AsyncBlock, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue, RegisterFile };
use peregrine_data::{Channel, PacketPriority, ProgramData, Region, Scale, ShapeRequest, StickId};
use serde_cbor::Value as CborValue;

simple_interp_command!(GetLaneInterpCommand,GetLaneDeserializer,21,3,(0,1,2));
simple_interp_command!(GetDataInterpCommand,GetDataDeserializer,22,6,(0,1,2,3,4,5));
simple_interp_command!(DataStreamInterpCommand,DataStreamDeserializer,23,3,(0,1,2));
simple_interp_command!(OnlyWarmInterpCommand,OnlyWarmDeserializer,43,1,(0));

impl InterpCommand for OnlyWarmInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let warm = get_instance::<bool>(context,"only_warm")?;
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Boolean(vec![warm]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for GetLaneInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let shape = get_instance::<ShapeRequest>(context,"request")?;
        let region = shape.region();
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Strings(vec![region.stick().get_id().to_string()]));
        registers.write(&self.1,InterpValue::Numbers(vec![region.index() as f64]));
        registers.write(&self.2,InterpValue::Numbers(vec![region.scale().get_index() as f64]));
        Ok(CommandResult::SyncResult())
    }
}

fn get_region(registers: &RegisterFile, cmd: &GetDataInterpCommand) -> anyhow::Result<Option<Region>> {
    if registers.len(&cmd.3)? == 0 { return Ok(None); }
    let stick = &registers.get_strings(&cmd.3)?[0];
    let index = &registers.get_numbers(&cmd.4)?[0];
    let scale = &registers.get_numbers(&cmd.5)?[0];
    Ok(Some(Region::new(&StickId::new(stick),*index as u64,&Scale::new(*scale as u64))))
}

async fn get(context: &mut InterpContext, cmd: GetDataInterpCommand) -> anyhow::Result<()> {
    let self_channel = get_instance::<Channel>(context,"channel")?;
    let program_data = get_instance::<ProgramData>(context,"data")?;
    let priority = get_instance::<PacketPriority>(context,"priority")?;
    let registers = context.registers_mut();
    let channel_name = registers.get_strings(&cmd.1)?;
    let prog_name = &registers.get_strings(&cmd.2)?[0];
    let mut ids = vec![];
    let mut net_time = None;
    if let Some(region) = get_region(registers,&cmd)? {
        drop(registers);
        let peregrine = get_peregrine(context)?;
        let data_store = peregrine.agent_store().data_store.clone();
        let channel = Channel::parse(&self_channel,&channel_name[0])?;
        let (result,took_ms) = data_store.get(&region,&channel,prog_name,&priority).await?;
        net_time = Some(took_ms);
        let id = program_data.add(result);
        ids.push(id as usize);
    }
    let registers = context.registers_mut();
    registers.write(&cmd.0,InterpValue::Indexes(ids));
    if let Some(net_time) = &mut net_time {
        let total_net_time = get_instance::<Arc<Mutex<f64>>>(context,"net_time")?;
        let mut total_net_time = total_net_time.lock().unwrap();
        *total_net_time += *net_time;
    }
    Ok(())
}

impl InterpCommand for GetDataInterpCommand {
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(get(context,cmd))))))
    }
}

impl InterpCommand for DataStreamInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let data_id = registers.get_indexes(&self.1)?[0];
        let names : Vec<String> = registers.get_strings(&self.2)?.iter().cloned().collect();
        drop(registers);
        let program_data = get_instance::<ProgramData>(context,"data")?;
        let data = program_data.get(data_id as u32)?;
        let mut out = vec![];
        for name in names {
            let values = data.get(&name)?;
            out.push(values.clone()); // XXX critical-path copy. Use Arc's to avoid, but involves significant changes in dauphin
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Bytes(out));
        Ok(CommandResult::SyncResult())
    }
}
