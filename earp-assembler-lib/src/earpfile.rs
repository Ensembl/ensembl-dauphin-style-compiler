use std::collections::HashMap;

use minicbor::{Encoder, Encode};

use crate::{command::{Operand, Command}, setmapper::SetMapper, suite::Suite, assets::Assets};

const EARPFILE_MAGIC : &str = "EARP0";

struct EntryPoints(HashMap<String,i64>);

impl<'t> Encode for EntryPoints {
    fn encode<W: minicbor::encode::Write>(&self, encoder: &mut Encoder<W>) -> Result<(), minicbor::encode::Error<W::Error>> {
        let mut ids = self.0.keys().collect::<Vec<_>>();
        ids.sort();
        encoder.begin_map()?;
        for id in ids {
            encoder.str(id)?.i64(*self.0.get(id).unwrap())?;
        }
        encoder.end()?;
        Ok(())
    }
}

pub struct EarpFileWriter<'t> {
    set_mapper: SetMapper<'t>,
    entry_points: EntryPoints,
    instructions: Vec<Command>,
    assets: Assets<'t>
}

impl<'t> EarpFileWriter<'t> {
    pub(crate) fn new(suite: &'t Suite) -> EarpFileWriter<'t> {
        EarpFileWriter {
            set_mapper: SetMapper::new(suite),
            entry_points: EntryPoints(HashMap::new()),
            instructions: vec![],
            assets: Assets::new(suite)
        }
    }

    pub(crate) fn assets_mut(&mut self) -> &mut Assets<'t> { &mut self.assets }
    pub(crate) fn set_mapper_mut(&mut self) -> &mut SetMapper<'t> { &mut self.set_mapper }

    pub(crate) fn add_instruction(&mut self, opcode: u64, operands: &[Operand]) {
        self.instructions.push(Command(opcode,operands.iter().cloned().collect()));
    }

    pub(crate) fn add_entry_point(&mut self, name: &str, pc: i64) {
        self.entry_points.0.insert(name.to_string(),pc);
    }

    #[cfg(test)]
    pub(crate) fn commands(&self) -> &[Command] { &self.instructions }

    #[cfg(test)]
    pub(crate) fn entry_points(&self) -> &HashMap<String,i64> { &self.entry_points.0 }
}

impl<'t> Encode for EarpFileWriter<'t> {
    fn encode<W: minicbor::encode::Write>(&self, encoder: &mut Encoder<W>) -> Result<(), minicbor::encode::Error<W::Error>> {
        encoder.begin_map()?
            .str("M")?.str(EARPFILE_MAGIC)?
            .str("S")?.encode(&self.set_mapper)?
            .str("E")?.encode(&self.entry_points)?
            .str("I")?.encode(&self.instructions)?
            .str("A")?.encode(&self.assets)?
            .end()?;
        Ok(())
    }
}
