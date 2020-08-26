use hashbrown::{ HashMap, HashSet };
use dauphin_interp::command::{ InterpLibRegister, CommandDeserializer, InterpCommand, CommandResult };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;

pub struct LookupInterpCommand(Register,Register,Register,Register,Register,Register);

impl InterpCommand for LookupInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let needles = registers.get_strings(&self.1)?;
        let haystack_offsets = registers.get_indexes(&self.3)?;
        let haystack_lens = registers.get_indexes(&self.4)?;
        let haystack_data = registers.get_strings(&self.2)?;
        let defaults = registers.get_indexes(&self.5)?;
        let num_haystacks = haystack_offsets.len();
        let mut outputs = vec![];
        for hs_index in 0..num_haystacks {
            let hs_start = haystack_offsets[hs_index];
            let hs_len = haystack_lens[hs_index];
            let default = defaults[hs_index%defaults.len()];
            let input : HashMap<&str,usize> = 
                haystack_data[hs_start..(hs_start+hs_len)].iter().enumerate().map(|(i,v)| {
                    (v as &str,i)
                }).collect();
            let output : Vec<usize> = 
                needles.iter().skip(hs_index).step_by(num_haystacks).map(|needle| {
                    *input.get(needle as &str).unwrap_or(&default)
                }).collect();
            outputs.push(output);
        }
        let out = if outputs.len() > 1 {
            let mut merged = vec![];
            let mut iters = outputs.iter().map(|x| x.iter()).collect::<Vec<_>>();
            let mut index = 0;
            while let Some(value) = iters[index].next() {
                index = (index+1) % outputs.len();
                merged.push(*value);
            }
            merged
        } else {
            outputs.remove(0)
        };
        registers.write(&self.0,InterpValue::Indexes(out));
        Ok(CommandResult::SyncResult())
    }
}

pub struct InInterpCommand(Register,Register,Register,Register,Register);

impl InterpCommand for InInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let needles = registers.get_strings(&self.1)?;
        let haystack_offsets = registers.get_indexes(&self.3)?;
        let haystack_lens = registers.get_indexes(&self.4)?;
        let haystack_data = registers.get_strings(&self.2)?;
        let num_haystacks = haystack_offsets.len();
        let mut outputs = vec![];
        for hs_index in 0..num_haystacks {
            let hs_start = haystack_offsets[hs_index];
            let hs_len = haystack_lens[hs_index];
            let input : HashSet<&str> = 
                haystack_data[hs_start..(hs_start+hs_len)].iter().map(|x| x as &str).collect();
            let output : Vec<bool> = 
                needles.iter().skip(hs_index).step_by(num_haystacks).map(|needle| {
                    input.contains(needle as &str)
                }).collect();
            outputs.push(output);
        }
        let out = if outputs.len() > 1 {
            let mut merged = vec![];
            let mut iters = outputs.iter().map(|x| x.iter()).collect::<Vec<_>>();
            let mut index = 0;
            while let Some(value) = iters[index].next() {
                index = (index+1) % outputs.len();
                merged.push(*value);
            }
            merged
        } else {
            outputs.remove(0)
        };
        registers.write(&self.0,InterpValue::Boolean(out));
        Ok(CommandResult::SyncResult())
    }
}

pub struct IndexInterpCommand(Register,Register,Register,Register,Register);

fn index_command<T>(dst: &mut Vec<T>, src: &[T], starts: &[usize],lengths: &[usize], needles: &[usize]) where T: Clone {
    for (offset,length) in starts.iter().zip(lengths.iter()) {
        for needle in needles {
            if needle >= length { break; }
            dst.push(src[offset+needle].clone());
        }
    }
}

impl InterpCommand for IndexInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let top_offset = registers.get_indexes(&self.1)?;
        let top_length = registers.get_indexes(&self.2)?;
        let needles = registers.get_indexes(&self.4)?;
        let src = registers.get(&self.3).borrow().get_shared()?;
        let natural = src.get_natural();
        let dst = InterpValue::Empty;
        let dst = dauphin_interp::polymorphic!(dst,[&src],natural,(|d,s| {
            index_command(d,s,&top_offset,&top_length,&needles)
        }));
        registers.write(&self.0,dst);
        Ok(CommandResult::SyncResult())
    }
}

pub struct LookupDeserializer();

impl CommandDeserializer for LookupDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((3,6))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(LookupInterpCommand(
            Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
            Register::deserialize(&value[2])?,Register::deserialize(&value[3])?,
            Register::deserialize(&value[4])?,Register::deserialize(&value[5])?)))
    }
}

pub struct InDeserializer();

impl CommandDeserializer for InDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((21,5))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(InInterpCommand(
            Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
            Register::deserialize(&value[2])?,Register::deserialize(&value[3])?,
            Register::deserialize(&value[4])?)))
    }
}

pub struct IndexDeserializer();

impl CommandDeserializer for IndexDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((22,5))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(IndexInterpCommand(
            Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
            Register::deserialize(&value[2])?,Register::deserialize(&value[3])?,
            Register::deserialize(&value[4])?)))
    }
}

pub(super) fn library_map_commands_interp(set: &mut InterpLibRegister) {
    set.push(LookupDeserializer());
    set.push(InDeserializer());
    set.push(IndexDeserializer());
}