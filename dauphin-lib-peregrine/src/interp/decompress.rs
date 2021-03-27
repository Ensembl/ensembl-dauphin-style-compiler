use anyhow::{ anyhow as err, bail };
use crate::simple_interp_command;
use dauphin_interp::command::{ CommandDeserializer, InterpCommand, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;
use inflate::inflate_bytes_zlib;
use std::str::from_utf8;

simple_interp_command!(InflateBytesInterpCommand,InflateBytesDeserializer,24,2,(0,1));
simple_interp_command!(InflateStringInterpCommand,InflateStringDeserializer,25,2,(0,1));
simple_interp_command!(Lesqlite2InterpCommand,Lesqlite2Deserializer,26,2,(0,1));
simple_interp_command!(ZigzagInterpCommand,ZigzagDeserializer,27,2,(0,1));
simple_interp_command!(DeltaInterpCommand,DeltaDeserializer,28,2,(0,1));
// 29 is unused
simple_interp_command!(ClassifyInterpCommand,ClassifyDeserializer,30,3,(0,1,2));
simple_interp_command!(SplitStringInterpCommand,SplitStringDeserializer,31,4,(0,1,2,3));

impl InterpCommand for InflateBytesInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let datas = registers.get_bytes(&self.1)?;
        let mut out = vec![];
        for data in datas.iter() {
            out.push(inflate_bytes_zlib(&data).map_err(|e| err!(e))?);

        }
        registers.write(&self.0,InterpValue::Bytes(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for InflateStringInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let datas = registers.get_bytes(&self.1)?;
        let mut out = vec![];
        for data in datas.iter() {
            let bytes = inflate_bytes_zlib(&data).map_err(|e| err!(e))?;
            let string = from_utf8(&bytes).map_err(|e| err!(e))?;
            out.push(string.to_string());
        }
        registers.write(&self.0,InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}

fn need_bytes(data: &[u8], i: &mut usize, n : usize) -> anyhow::Result<()> {
    if *i+n > data.len() {
        bail!("premature termination of stream")
    }
    *i += n;
    Ok(())
}

impl InterpCommand for Lesqlite2InterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let datas = registers.get_bytes(&self.1)?;
        let mut out = vec![];
        for data in datas.iter() {
            let mut i = 0;
            while i < data.len() {
                if data[i] < 178 {
                    need_bytes(data,&mut i,1)?;
                    out.push(data[i-1] as f64);
                } else if data[i] < 242 {
                    need_bytes(data,&mut i,2)?;
                    out.push((((data[i-2] as u64-178)<<8) + (data[i-1] as u64) + 178_u64) as f64);
                } else if data[i] < 250 {
                    need_bytes(data,&mut i,3)?;
                    let v : u64 = 
                        ((data[i-3] as u64-242_u64)<<16_u64) +
                        ((data[i-1] as u64) << 8_u64) +
                        (data[i-2] as u64) +
                        16562_u64
                    ;
                    out.push(v as f64);
                } else {
                    need_bytes(data,&mut i,1)?;
                    let n = (data[i-1] - 247) as usize;
                    need_bytes(data,&mut i,n)?;
                    let mut v = 0;
                    let mut m = 0;
                    for j in 0..n {
                        v += (data[i-n+j] as u64) << m;
                        m += 8;
                    }
                    out.push(v as f64);
                }
            }
        }
        registers.write(&self.0,InterpValue::Numbers(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ZigzagInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let data = registers.get_numbers(&self.1)?;
        let mut out = vec![];
        for v in data.iter() {
            let v = *v as u64;
            if v%2 == 1 {
                out.push((-((v as i64)+1)/2) as f64);
            } else {
                out.push(((v as i64)/2) as f64);
            }
        }
        registers.write(&self.0,InterpValue::Numbers(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for DeltaInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let data = registers.get_numbers(&self.1)?;
        let mut out = vec![];
        let mut prev : i64 = 0;
        for v in data.iter() {
            let v = *v as i64;
            prev += v;
            out.push(prev as f64);
            
        }
        registers.write(&self.0,InterpValue::Numbers(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for ClassifyInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let keys = registers.get_strings(&self.1)?;
        let values = registers.get_indexes(&self.2)?;
        let mut out = vec![];
        for v in values.iter() {
            out.push(keys.get(*v).ok_or_else(|| err!("bad index in classify"))?.to_string());
        }
        registers.write(&self.0,InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}

impl InterpCommand for SplitStringInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let strings = registers.get_strings(&self.3)?;
        let mut out_offset = vec![];
        let mut out_length = vec![];
        let mut out_data = vec![];
        for string in strings.iter() {
            out_offset.push(out_data.len());
            let mut more : Vec<_> = string.split("\0").map(|x| x.to_string()).collect();
            out_length.push(more.len());
            out_data.append(&mut more);

        }
        registers.write(&self.0,InterpValue::Strings(out_data));
        registers.write(&self.1,InterpValue::Indexes(out_offset));
        registers.write(&self.2,InterpValue::Indexes(out_length));
        Ok(CommandResult::SyncResult())
    }
}
