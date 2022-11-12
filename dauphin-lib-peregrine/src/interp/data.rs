use std::sync::Mutex;
use std::sync::Arc;
use crate::simple_interp_command;
use crate::util::get_peregrine;
use crate::util::{ get_instance };
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, AsyncBlock, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use peregrine_data::AccessorResolver;
use peregrine_data::DataMessage;
use peregrine_data::DataRequest;
use peregrine_data::ObjectBuilder;
use peregrine_data::LoadMode;
use peregrine_data::ReceivedData;
use peregrine_data::RunReport;
use peregrine_data::{PacketPriority, Region, Scale, ShapeRequest, StickId};
use peregrine_toolkit::lock;
use serde_cbor::Value as CborValue;
use anyhow::anyhow as error;

simple_interp_command!(DataNumberInterpCommand,DataNumberDeserializer,6,3,(0,1,2));
simple_interp_command!(DataStringInterpCommand,DataStringDeserializer,8,3,(0,1,2));
simple_interp_command!(DataBooleanInterpCommand,DataBooleanDeserializer,11,3,(0,1,2));
simple_interp_command!(GetLaneInterpCommand,GetLaneDeserializer,21,3,(0,1,2));
simple_interp_command!(GetDataInterpCommand,GetDataDeserializer,22,2,(0,1));
simple_interp_command!(DataStreamInterpCommand,DataStreamDeserializer,23,3,(0,1,2));
simple_interp_command!(OnlyWarmInterpCommand,OnlyWarmDeserializer,43,1,(0));
simple_interp_command!(RequestInterpCommand,RequestDeserializer,10,6,(0,1,2,3,4,5));
simple_interp_command!(RequestScopeInterpCommand,RequestScopeDeserializer,52,4,(0,1,2,3));
simple_interp_command!(MakeRegionInterpCommand,MakeRegionDeserializer,75,6,(0,1,2,3,4,5));

impl InterpCommand for OnlyWarmInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let mode = get_instance::<LoadMode>(context,"mode")?;
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Boolean(vec![!mode.build_shapes()]));
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

async fn get(context: &mut InterpContext, cmd: GetDataInterpCommand) -> anyhow::Result<()> {
    //let program_data = get_instance::<ProgramData>(context,"data")?;
    let mode = get_instance::<LoadMode>(context,"mode")?;
    let priority = if mode.high_priority() { PacketPriority::RealTime } else { PacketPriority::Batch };
    let registers = context.registers();
    let request_id = registers.get_indexes(&cmd.1)?[0] as u32;
    drop(registers);
    let geometry = get_instance::<ObjectBuilder>(context,"builder")?;
    let peregrine = get_peregrine(context)?;
    let data_store = peregrine.agent_store().data_store.clone();
    let request = geometry.request(request_id)?;
    let (result,took_ms) = data_store.get(&request,&priority).await.map_err(|e| DataMessage::XXXTransitional(e))?;
    let data_id = geometry.add_data(result);
    drop(peregrine);
    let registers = context.registers_mut();
    registers.write(&cmd.0,InterpValue::Indexes(vec![data_id as usize]));
    let total_net_time = get_instance::<Arc<Mutex<RunReport>>>(context,"report")?;
    lock!(total_net_time).net_ms += took_ms;
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
        let geometry = get_instance::<ObjectBuilder>(context,"builder")?;
        let data = geometry.data(data_id as u32)?.as_ref().clone();
        let mut out = vec![];
        for name in names {
            let values = data.get(&name)?;
            out.push(values.data_as_bytes().map_err(|_| error!("not bytes"))?.to_vec()); // XXX critical-path copy. Use Arc's to avoid, but involves significant changes in dauphin
        }
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Bytes(out));
        Ok(CommandResult::SyncResult())
    }
}

fn get_data<'a,F,G,X>(context: &mut InterpContext, reg0: &Register, reg1: &Register, reg2: &Register, cb_each: F, cb_all: G) -> anyhow::Result<CommandResult<'a>>
        where F: Fn(&ReceivedData) -> Result<X,()>, G: FnOnce(Vec<X>) -> InterpValue {
    let registers = context.registers_mut();
    let data_id = registers.get_indexes(reg1)?[0];
    let names : Vec<String> = registers.get_strings(reg2)?.iter().cloned().collect();
    drop(registers);
    let geometry = get_instance::<ObjectBuilder>(context,"builder")?;
    let data = geometry.data(data_id as u32)?.as_ref().clone();
    let mut out = vec![];
    for name in names {
        let values = data.get2(&name)?;
        out.push(cb_each(values).map_err(|_| error!("unecpected type"))?); // XXX critical-path copy. Use Arc's to avoid, but involves significant changes in dauphin
    }
    let registers = context.registers_mut();
    registers.write(reg0,cb_all(out));
    Ok(CommandResult::SyncResult())
}

impl InterpCommand for DataNumberInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        get_data(context,&self.0,&self.1,&self.2, move |data| {
            data.data_as_numbers().cloned()
        },|mut data| {
            let mut out = vec![];
            for more in data.drain(..) {
                out.extend(more.iter());
            }
            InterpValue::Numbers(out)
        })
    }
}

impl InterpCommand for DataStringInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        get_data(context,&self.0,&self.1,&self.2, move |data| {
            data.data_as_strings().cloned()
        },|mut data| {
            let mut out = vec![];
            for more in data.drain(..) {
                out.extend(more.iter().cloned());
            }
            InterpValue::Strings(out)
        })
    }
}

impl InterpCommand for DataBooleanInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        get_data(context,&self.0,&self.1,&self.2, move |data| {
            data.data_as_booleans().cloned()
        },|mut data| {
            let mut out = vec![];
            for more in data.drain(..) {
                out.extend(more.iter());
            }
            InterpValue::Boolean(out)
        })
    }
}

async fn request_interp_command(context: &mut InterpContext, cmd: RequestInterpCommand) -> anyhow::Result<()> {
    let channel_resolver = get_instance::<AccessorResolver>(context,"channel-resolver")?;
    let registers = context.registers_mut();
    let channel_name = registers.get_strings(&cmd.1)?[0].to_owned();
    let prog_name = registers.get_strings(&cmd.2)?[0].to_owned();
    let stick = &registers.get_strings(&cmd.3)?[0];
    let index = &registers.get_numbers(&cmd.4)?[0];
    let scale = &registers.get_numbers(&cmd.5)?[0];
    let region = Region::new(&StickId::new(stick),*index as u64,&Scale::new(*scale as u64));
    let channel = channel_resolver.resolve(&channel_name).await.map_err(|e| DataMessage::XXXTransitional(e))?;
    let request = DataRequest::new(&channel,&prog_name,&region);
    drop(registers);
    let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
    let id = geometry_builder.add_request(request);
    let registers = context.registers_mut();
    registers.write(&cmd.0,InterpValue::Indexes(vec![id as usize]));
    Ok(())
}

impl InterpCommand for RequestInterpCommand {
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let cmd = self.clone();
        Ok(CommandResult::AsyncResult(AsyncBlock::new(Box::new(|context| Box::pin(request_interp_command(context,cmd))))))
    }
}

impl InterpCommand for RequestScopeInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let request_id = registers.get_indexes(&self.1)?[0] as u32;
        let key = registers.get_strings(&self.2)?[0].to_owned();
        let values = registers.get_strings(&self.3)?.to_vec();
        drop(registers);
        let geometry_builder = get_instance::<ObjectBuilder>(context,"builder")?;
        let request = geometry_builder.request(request_id)?;
        let request = request.add_scope(&key,&values);
        let new_id = geometry_builder.add_request(request);
        let registers = context.registers_mut();
        registers.write(&self.0,InterpValue::Indexes(vec![new_id as usize]));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for MakeRegionInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        /* get data */
        let stick = registers.get_strings(&self.3)?[0].to_string();
        let start = registers.get_numbers(&self.4)?[0];
        let end = registers.get_numbers(&self.5)?[0];
        let scale = Scale::new_bp_per_screen(end-start);
        let index = scale.carriage((start+end)/2.);
        /* return region */
        registers.write(&self.0,InterpValue::Strings(vec![stick.to_string()]));
        registers.write(&self.1,InterpValue::Numbers(vec![index as f64]));
        registers.write(&self.2,InterpValue::Numbers(vec![scale.get_index() as f64]));
        Ok(CommandResult::SyncResult())
    }
}
