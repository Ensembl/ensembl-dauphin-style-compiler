use peregrine_toolkit::{diffset::DiffSet, eachorevery::eoestruct::{StructValue}, error::Error, lengths_match, multizip};
use crate::{shapeload::programname::ProgramName};
use super::programspec::{ProgramModel, ProgramSetting, ProgramModelBuilder};

fn lookup<T>(index: usize, array: &[T]) -> Result<&T,Error> {
    array.get(index).ok_or_else(|| Error::operr("bad track packet"))
}

struct PackedProgram {
    name: usize,
    in_bundle_name: usize,
    set: usize,
    version: usize,
    defaults: Vec<(usize,usize)>
}

impl PackedProgram {
    fn to_program_model(&self, res: &PackedProgramSpec) -> Result<ProgramModel,Error> {
        let program_name = ProgramName::new(
            lookup(self.set,&res.name_idx)?,
            lookup(self.name,&res.name_idx)?,
            self.version as u32
        );
        let mut model = ProgramModelBuilder::new(
            &program_name,
            lookup(self.in_bundle_name,&res.name_idx)?
        );
        for (setting_name,default) in &self.defaults {
            let name = lookup(*setting_name,&res.key_idx)?;
            let default = lookup(*default,&res.value_idx)?;
            let setting = ProgramSetting::new(name,default.clone());
            model.add_setting(name,setting);
        }
        Ok(ProgramModel::new(model))
    }
}

#[derive(serde_derive::Deserialize)]
pub(crate) struct PackedProgramSpec {
    /* tracks */
    name: DiffSet,
    in_bundle_name: DiffSet,
    set: DiffSet,
    version: DiffSet,
    keys: Vec<DiffSet>,
    defaults: Vec<DiffSet>,    

    /* indexes all to the above */
    name_idx: Vec<String>,
    key_idx: Vec<String>,
    value_idx: Vec<StructValue>,
}

impl PackedProgramSpec {
    fn make_packed_programs(&self) -> Result<Vec<PackedProgram>,Error> {
        let mut out = vec![];
        if !lengths_match!(self,
            name,in_bundle_name,set,version, keys, defaults
        ) {
            return Err(Error::operr("Bad packet: lengths don't match"));
        }
        multizip!(self;
            name,in_bundle_name,set,version,keys,defaults; {
                if !lengths_match!(self,keys,defaults) {
                    return Err(Error::operr("Bad packet: lengths don't match"));
                }
                let mut default_data = vec![];
                multizip!(keys,defaults; {
                    default_data.push((keys,defaults));
                });
                out.push(PackedProgram {
                    name, in_bundle_name, set, version,
                    defaults: default_data
                });
        });
        Ok(out)
    }

    pub(crate) fn to_program_models(&self) -> Result<Vec<ProgramModel>,Error> {
        self.make_packed_programs()?.iter().map(|x| x.to_program_model(&self)).collect()
    }
}
