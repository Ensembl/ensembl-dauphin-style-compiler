use std::collections::HashMap;
use dauphin_interp::command::{ InterpLibRegister, CommandDeserializer, InterpCommand };
use dauphin_interp::runtime::{ InterpContext, Register, InterpValue };
use serde_cbor::Value as CborValue;

pub struct LookupInterpCommand(Register,Register,Register,Register,Register,Register);

impl InterpCommand for LookupInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<()> {
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
            let input : HashMap<String,usize> = 
                haystack_data[hs_start..(hs_start+hs_len)].iter().enumerate().map(|(i,v)| {
                    (v.to_string(),i)
                }).collect();
            let output : Vec<usize> = 
                needles.iter().skip(hs_index).step_by(num_haystacks).map(|needle| {
                    *input.get(needle).unwrap_or(&default)
                }).collect();
            outputs.push(output);
        }
        let mut merged = vec![];
        let mut iters = outputs.iter().map(|x| x.iter()).collect::<Vec<_>>();
        let mut index = 0;
        while let Some(value) = iters[index].next() {
            index = (index+1) % outputs.len();
            merged.push(*value);
        }
        registers.write(&self.0,InterpValue::Indexes(merged));
        Ok(())
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

pub(super) fn library_map_commands_interp(set: &mut InterpLibRegister) {
    set.push(LookupDeserializer());
}