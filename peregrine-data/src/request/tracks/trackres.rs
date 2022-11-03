use peregrine_toolkit::{error::Error};
use super::{ trackmodel::{TrackModel}, expansionmodel::{ExpansionModel}, packedtrackres::PackedTrackRes };

pub(crate) enum TrackResult {
    Packed(PackedTrackRes),
    Unpacked(Vec<TrackModel>,Vec<ExpansionModel>),
    None
}

impl TrackResult {
    pub(crate) async fn to_track_models(&self) -> Result<(Vec<TrackModel>,Vec<ExpansionModel>),Error> {
        Ok(match self {
            TrackResult::Packed(p) => (p.to_track_models().await?,p.to_expansion_models()?),
            TrackResult::Unpacked(t,e) => (t.to_vec(),e.to_vec()),
            TrackResult::None => (vec![],vec![])
        })
    }
}
