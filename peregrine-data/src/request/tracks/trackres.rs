use peregrine_toolkit::{error::Error};
use crate::{ BackendNamespace};
use super::{ trackmodel::{TrackModel}, expansionmodel::{ExpansionModel}, packedtrackres::PackedTrackRes };

pub(crate) enum TrackResult {
    Packed(PackedTrackRes),
    Unpacked(Vec<TrackModel>,Vec<ExpansionModel>)
}

impl TrackResult {
    pub(crate) fn to_track_models(self, backend_namespace: &BackendNamespace) -> Result<(Vec<TrackModel>,Vec<ExpansionModel>),Error> {
        Ok(match self {
            TrackResult::Packed(mut p) => (p.to_track_models(backend_namespace)?,p.to_expansion_models()?),
            TrackResult::Unpacked(t,e) => (t,e)
        })
    }
}
