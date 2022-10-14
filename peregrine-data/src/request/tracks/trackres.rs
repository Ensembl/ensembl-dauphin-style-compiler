use peregrine_toolkit::{error::Error, log};
use super::{ diffset::DiffSet, switchtree::SwitchTree, trackmodel::{TrackModel, TrackModelBuilder} };

#[derive(Debug)]
struct PackedTrack {
    name: String,
    program: usize,
    tags: Vec<usize>,
    triggers: Vec<usize>,
    extra: Vec<usize>,
    set: Vec<usize>,
    scale_start: usize,
    scale_end: usize,
    scale_step: usize
}

fn lookup<T>(index: usize, array: &[T]) -> Result<&T,Error> {
    array.get(index).ok_or_else(|| Error::operr("bad track packet"))
}

impl PackedTrack {
    fn to_track(&self, res: &PackedTrackRes) -> Result<TrackModel,Error> {
        let program = lookup(self.program,&res.program_idx)?;
        let mut builder = TrackModelBuilder::new(&self.name,program,self.scale_start,self.scale_end,self.scale_step);
        for tag_idx in &self.tags {
            builder.add_tag(lookup(*tag_idx,&res.tag_idx)?);
        }
        for trigger_idx in &self.triggers {
            builder.add_trigger(lookup(*trigger_idx,&res.switch_idx.0)?);
        }
        for extra_idx in &self.extra {
            builder.add_extra(lookup(*extra_idx,&res.switch_idx.0)?);
        }
        for set_idx in &self.set {
            builder.add_set(lookup(*set_idx,&res.switch_idx.0)?);
        }
        Ok(TrackModel::new(builder))
    }
}

#[derive(serde_derive::Deserialize,Debug)]
pub(crate) struct PackedTrackRes {
    name: Vec<String>,
    program: Vec<usize>,
    tags: Vec<DiffSet>,
    triggers: Vec<DiffSet>,
    extra: Vec<DiffSet>,
    set: Vec<DiffSet>,
    scale_start: Vec<usize>,
    scale_end: Vec<usize>,
    scale_step: Vec<usize>,
    switch_idx: SwitchTree,
    program_idx: Vec<String>,
    tag_idx: Vec<String>
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
        log!("{:?}",self);
        let mut out = vec![];
        if !lengths_match!(self,name,program,tags,triggers,extra,set,scale_start,scale_end,scale_step) {
            return Err(Error::operr("Bad packet: lengths don't match"));
        }
        multizip!(self,name,program,tags,triggers,extra,set,scale_start,scale_end,scale_step;{
            out.push(PackedTrack {
                name, program,scale_start,scale_end,scale_step,
                tags: tags.0,
                triggers: triggers.0,
                extra: extra.0,
                set: set.0,
            });
        });
        log!("{:?}",out);
        Ok(out)
    }

    fn to_track_models(self) -> Result<Vec<TrackModel>,Error> {
        self.make_packed_tracks()?.drain(..).map(|t| t.to_track(&self)).collect()
    }
}

pub(crate) enum TrackResult {
    Packed(PackedTrackRes),
    Unpacked(Vec<TrackModel>)
}

impl TrackResult {
    pub(crate) fn to_track_models(self) -> Result<Vec<TrackModel>,Error> {
        Ok(match self {
            TrackResult::Packed(p) => p.to_track_models()?,
            TrackResult::Unpacked(u) => u
        })
    }
}