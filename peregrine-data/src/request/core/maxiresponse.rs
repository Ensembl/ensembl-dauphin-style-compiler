use peregrine_toolkit::{ serdetools::{st_field }, log};
use serde::de::{Visitor, MapAccess, DeserializeSeed, IgnoredAny};
use serde::{Deserializer};
use std::any::Any;
use std::fmt;
use std::mem::replace;
use std::sync::Arc;
use crate::{core::{channel::wrappedchannelsender::WrappedChannelSender, program::programbundle::PackedSuppliedBundle }, request::tracks::{trackres::TrackResult, packedtrackres::PackedTrackRes}, TrackModel, ExpansionModel};
use crate::{BackendNamespace};
use crate::core::program::programbundle::SuppliedBundle;
use super::miniresponse::{MiniResponseAttempt, MiniResponseAttemptVecDeserialize};

#[allow(unused)] // used in debug_big_requests
use peregrine_toolkit::warn;

#[allow(unused)] // used in debug_big_requests
const TOO_LARGE : usize = 100*1024;

pub struct MaxiResponse {
    channel: BackendNamespace,
    responses: Vec<MiniResponseAttempt>,
    programs: Vec<SuppliedBundle>,
    tracks: Vec<TrackResult>
}

impl MaxiResponse {
    pub fn empty(channel: &BackendNamespace) -> MaxiResponse {
        MaxiResponse {
            channel: channel.clone(),
            responses: vec![],
            programs: vec![],
            tracks: vec![TrackResult::None]
        }
    }

    pub fn add_response(&mut self, response: MiniResponseAttempt) {
        self.responses.push(response);
    }

    pub fn add_track_payload(&mut self, tracks: Vec<TrackModel>, expansions: Vec<ExpansionModel>) {
        self.tracks.push(TrackResult::Unpacked(tracks,expansions));
    }

    pub fn set_bundle_payload(&mut self, bundles: Vec<SuppliedBundle>) {
        self.programs = bundles;
    }

    #[cfg(debug_big_requests)]
    fn check_big_requests(&self) {
        let total_size : usize = self.responses.iter().map(|x| x.total_size()).sum();
        if total_size > TOO_LARGE {
            warn!("excessively large maxi-response {} ({} elements)",total_size,self.responses.len());
        }
        for mini in &self.responses {
            if mini.total_size() > TOO_LARGE/5 {
                warn!("excessively large mini-response {}",mini.description());
                for (key,size) in mini.component_size().iter() {
                    if *size > TOO_LARGE/15 {
                        warn!("excessively large mini-response internal key {} ({})",key,size);
                    }
                }
            }
        }
    }

    #[cfg(not(debug_big_requests))]
    fn check_big_requests(&self) {}

    pub(crate) fn channel(&self) -> &BackendNamespace { &self.channel }
    pub(crate) fn programs(&self) -> &[SuppliedBundle] { &self.programs }
    pub(crate) fn tracks(&self) -> &[TrackResult] { &self.tracks }
    pub(crate) fn take_responses(&mut self) -> Vec<MiniResponseAttempt> {
        self.check_big_requests();
        replace(&mut self.responses,vec![])
    }
}

struct MaxiResponseVisitor(WrappedChannelSender,Arc<dyn Any>);

impl<'de> Visitor<'de> for MaxiResponseVisitor {
    type Value = MaxiResponse;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a MaxiResponse")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut responses : Option<Vec<MiniResponseAttempt>> = None;
        let mut programs = None;
        let mut channel = None;
        let mut tracks = vec![TrackResult::None];
        while let Some(key) = access.next_key()? {
            match key {
                "responses" => { 
                    responses = Some(access.next_value_seed(MiniResponseAttemptVecDeserialize(self.0.clone(),self.1.clone()))?);
                },
                "programs" => { programs = access.next_value::<Option<Vec<PackedSuppliedBundle>>>()? },
                "channel" => { channel = access.next_value()? },
                "tracks-packed" => { 
                    tracks = access.next_value::<Vec<PackedTrackRes>>()?.drain(..)
                        .map(|x| TrackResult::Packed(x)).collect();
                },
                _ => { let _ : IgnoredAny = access.next_value()?; }
            }
        }
        let responses = st_field("responses",responses)?;
        let channel = st_field("channel",channel)?;
        let mut programs = st_field("programs",programs)?;
        let programs = programs.drain(..).map(|x| x.0).collect::<Vec<_>>();
        Ok(MaxiResponse {
            channel, 
            responses, 
            programs,
            tracks
        })
    }
}

pub struct MaxiResponseDeserialize(pub(crate) WrappedChannelSender,pub(crate) Arc<dyn Any>);

impl<'de> DeserializeSeed<'de> for MaxiResponseDeserialize {
    type Value = MaxiResponse;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(MaxiResponseVisitor(self.0.clone(),self.1.clone()))
    }
}
