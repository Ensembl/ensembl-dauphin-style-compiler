use peregrine_toolkit::{error::Error, eachorevery::eoestruct::StructBuilt };
use crate::{ProgramName, BackendNamespace};
use super::{ diffset::DiffSet, switchtree::SwitchTree, trackmodel::{TrackModel, TrackModelBuilder}, expansionmodel::{ExpansionModel, ExpansionModelBuilder} };

#[derive(Debug)]
struct PackedTrack {
    name: String,
    program: usize,
    tags: Vec<usize>,
    triggers: Vec<usize>,
    extra: Vec<usize>,
    set: Vec<usize>,
    values: Vec<usize>,
    settings_keys: Vec<usize>,
    settings_values: Vec<usize>,
    values_keys: Vec<usize>,
    values_values: Vec<usize>,
    scale_start: u64,
    scale_end: u64,
    scale_step: u64,
}

fn lookup<T>(index: usize, array: &[T]) -> Result<&T,Error> {
    array.get(index).ok_or_else(|| Error::operr("bad track packet"))
}

impl PackedTrack {
    fn to_track(&self, backend_namespace: &BackendNamespace, res: &PackedTrackRes) -> Result<TrackModel,Error> {
        let program = lookup(self.program,&res.program_idx)?;
        let program_name = ProgramName(backend_namespace.clone(),program.clone());
        let mut builder = TrackModelBuilder::new(&self.name,&program_name,self.scale_start,self.scale_end,self.scale_step);
        for tag_idx in &self.tags {
            builder.add_tag(lookup(*tag_idx,&res.tag_idx)?);
        }
        for trigger_idx in &self.triggers {
            builder.add_trigger(lookup(*trigger_idx,&res.switch_idx.0)?);
        }
        for extra_idx in &self.extra {
            builder.add_extra(lookup(*extra_idx,&res.switch_idx.0)?);
        }
        for (set_idx,value_idx) in self.set.iter().zip(&self.values) {
            builder.add_set(
                lookup(*set_idx,&res.switch_idx.0)?,
                lookup(*value_idx,&res.value_idx)?.clone(),
            );
        }
        for (key_idx,switch_idx) in self.settings_keys.iter().zip(&self.settings_values) {
            builder.add_setting(
                lookup(*key_idx,&res.key_idx)?,
                lookup(*switch_idx,&res.switch_idx.0)?
            );
        }
        for (key_idx,value_idx) in self.values_keys.iter().zip(&self.values_values) {
            builder.add_value(
                lookup(*key_idx,&res.key_idx)?,
                lookup(*value_idx,&res.value_idx)?.clone(),
            );
        }
        Ok(TrackModel::new(builder))
    }
}

#[derive(Debug)]
struct PackedExpansion {
    name: String,
    channel: usize,
    triggers: Vec<usize>    
}

impl PackedExpansion {
    fn to_expansion(&self, res: &PackedTrackRes) -> Result<ExpansionModel,Error> {
        let bn_name = lookup(self.channel,&res.channel_idx.0)?;
        if bn_name.len() != 2 {
            return Err(Error::operr("bad track model payload"));
        }
        let backend_namespace = BackendNamespace::new(&bn_name[0],&bn_name[1]);
        let mut builder = ExpansionModelBuilder::new(&backend_namespace,&self.name);
        for trigger_idx in &self.triggers {
            builder.add_trigger(lookup(*trigger_idx,&res.switch_idx.0)?);
        }
        Ok(ExpansionModel::new(builder))
    }
}

#[derive(serde_derive::Deserialize,Debug)]
pub(crate) struct PackedTrackRes {
    /* tracks */
    name: Vec<String>,
    program: Vec<usize>,
    tags: Vec<DiffSet>,
    triggers: Vec<DiffSet>,
    extra: Vec<DiffSet>,
    set: Vec<DiffSet>,
    scale_start: Vec<u64>,
    scale_end: Vec<u64>,
    scale_step: Vec<u64>,
    values: Vec<Vec<usize>>,
    #[serde(rename = "values-keys")]
    values_keys: Vec<DiffSet>,
    #[serde(rename = "values-values")]
    values_values: Vec<Vec<usize>>,
    #[serde(rename = "settings-keys")]
    settings_keys: Vec<DiffSet>,
    #[serde(rename = "settings-values")]
    settings_values: Vec<Vec<usize>>,

    /* expansions */
    #[serde(rename = "e-name")]
    e_name: Vec<String>,
    #[serde(rename = "e-channel")]
    e_channel: DiffSet,
    #[serde(rename = "e-triggers")]
    e_triggers: Vec<DiffSet>,

    /* indexes all to the above */
    switch_idx: SwitchTree,
    program_idx: Vec<String>,
    tag_idx: Vec<String>,
    channel_idx: SwitchTree,
    value_idx: Vec<StructBuilt>,
    key_idx: Vec<String>
}

macro_rules! lengths_match {
    ($self:expr,$first:ident,$($rest:ident),*) => {
        (|| {
            let len = $self.$first.len();
            $( if $self.$rest.len() != len { return false; } )*
            true
        })()
    }
}

macro_rules! multizip {
    ($self:expr,$($arg:ident),*;$cb:expr) => {
        {
            use itertools::izip;

            for ($($arg),*) in izip!($($self.$arg.iter().cloned()),*) {
                $cb
            }
        }
    }
}

impl PackedTrackRes {
    fn make_packed_tracks(&self) -> Result<Vec<PackedTrack>,Error> {
        let mut out = vec![];
        if !lengths_match!(self,
            name,program,tags,triggers,extra,set,scale_start,scale_end,scale_step,values,
            values_keys,values_values,settings_keys,settings_values
        ) {
            return Err(Error::operr("Bad packet: lengths don't match"));
        }
        multizip!(self,
            name,program,tags,triggers,extra,set,scale_start,scale_end,scale_step,values,
            values_keys,values_values,settings_keys,settings_values;
            {
            out.push(PackedTrack {
                name, program,scale_start,scale_end,scale_step,
                tags: tags.0,
                triggers: triggers.0,
                extra: extra.0,
                set: set.0,
                values: values,
                settings_keys: settings_keys.0,
                settings_values,
                values_keys: values_keys.0,
                values_values
            });
        });
        Ok(out)
    }

    fn make_packed_expansions(&self) -> Result<Vec<PackedExpansion>,Error> {
        let mut out = vec![];
        if !lengths_match!(self,name,program,tags,triggers,extra,set,scale_start,scale_end,scale_step) {
            return Err(Error::operr("Bad packet: lengths don't match"));
        }
        multizip!(self,e_name,e_channel,e_triggers;{
            out.push(PackedExpansion {
                name: e_name,
                channel: e_channel,
                triggers: e_triggers.0
            });
        });
        Ok(out)
    }

    pub(super) fn to_track_models(&mut self, backend_namespace: &BackendNamespace) -> Result<Vec<TrackModel>,Error> {
        self.make_packed_tracks()?.drain(..).map(|t| t.to_track(backend_namespace,&self)).collect()
    }

    pub(super) fn to_expansion_models(&mut self) -> Result<Vec<ExpansionModel>,Error> {
        self.make_packed_expansions()?.drain(..).map(|t| t.to_expansion(&self)).collect()
    }    
}
