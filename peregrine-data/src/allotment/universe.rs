use std::sync::{Arc, Mutex};

use crate::api::PlayingField;
use crate::{AllotmentMetadata, AllotmentMetadataReport, AllotmentMetadataStore, AllotmentRequest, CoordinateSystem};
use peregrine_toolkit::lock;

use super::baseallotmentrequest::trim_prefix;
use super::lineargroup::lineargroup::{LinearGroup};
use super::lineargroup::offsetbuilder::LinearOffsetBuilder;
use super::lineargroup::secondary::SecondaryPositionStore;
use super::{dustbinallotment::DustbinAllotmentRequest,  maintrack::MainTrackRequestCreator, offsetallotment::OffsetAllotmentRequestCreator};

struct UniverseData {
    dustbin: Arc<DustbinAllotmentRequest>,
    main: LinearGroup<MainTrackRequestCreator>,
    top_tracks: LinearGroup<MainTrackRequestCreator>,
    bottom_tracks: LinearGroup<MainTrackRequestCreator>,
    window_tracks: LinearGroup<OffsetAllotmentRequestCreator>,
    window_tracks_bottom: LinearGroup<OffsetAllotmentRequestCreator>,
    left: LinearGroup<OffsetAllotmentRequestCreator>,
    right: LinearGroup<OffsetAllotmentRequestCreator>,
    window: LinearGroup<OffsetAllotmentRequestCreator>,
    window_bottom: LinearGroup<OffsetAllotmentRequestCreator>,
    playingfield: PlayingField
}

impl UniverseData {
    fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        if name == "" {
            Some(AllotmentRequest::upcast(self.dustbin.clone()))
        } else if let Some(suffix) = trim_prefix("track",name) {
            self.main.make_request(allotment_metadata,&suffix,&name)
        } else if let Some(suffix) = trim_prefix("track-top",name) {
            self.top_tracks.make_request(allotment_metadata,&suffix,&name)
        } else if let Some(suffix) = trim_prefix("track-bottom",name) {
            self.bottom_tracks.make_request(allotment_metadata,&suffix,&name)
        } else if let Some(suffix) = trim_prefix("track-window",name) {
            self.window_tracks.make_request(allotment_metadata,&suffix,&name)
        } else if let Some(suffix) = trim_prefix("window",name) {
            self.window.make_request(allotment_metadata,&suffix,&name)
        } else if let Some(suffix) = trim_prefix("window-bottom",name) {
            self.window_bottom.make_request(allotment_metadata,&suffix,&name)
        } else if let Some(suffix) = trim_prefix("track-window-bottom",name) {
            self.window_tracks_bottom.make_request(allotment_metadata,&suffix,&name)
        } else if let Some(suffix) = trim_prefix("left",name) {
            self.left.make_request(allotment_metadata,&suffix,&name)
        } else if let Some(suffix) = trim_prefix("right",name) {
            self.right.make_request(allotment_metadata,&suffix,&name)
        } else {
            None
        }
    }

    fn union(&mut self, other: &UniverseData) {
        self.top_tracks.union(&other.top_tracks);
        self.bottom_tracks.union(&other.bottom_tracks);
        self.window_tracks.union(&other.window_tracks);
        self.window_tracks_bottom.union(&other.window_tracks_bottom);
        self.window_bottom.union(&other.window_bottom);
        self.main.union(&other.main);
        self.window.union(&other.window);
        self.left.union(&other.left);
        self.right.union(&other.right);
    }

    fn get_all_metadata(&self,allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        self.main.get_all_metadata(allotment_metadata,out);
        self.top_tracks.get_all_metadata(allotment_metadata,out);
        self.bottom_tracks.get_all_metadata(allotment_metadata,out);
        self.window.get_all_metadata(allotment_metadata,out);
        self.window_bottom.get_all_metadata(allotment_metadata,out);
        self.window_tracks.get_all_metadata(allotment_metadata,out);
        self.window_tracks_bottom.get_all_metadata(allotment_metadata,out);
        self.left.get_all_metadata(allotment_metadata,out);
        self.right.get_all_metadata(allotment_metadata,out);
    }

    fn allot(&mut self) {
        /* Left and Right */
        let mut secondary = SecondaryPositionStore::new();
        let mut left_offset = LinearOffsetBuilder::new();
        self.left.allot(0,&mut left_offset,&mut secondary);
        let mut right_offset = LinearOffsetBuilder::new();
        self.right.allot(0,&mut right_offset, &mut secondary);
        let left = self.left.max();
        /* Main run */
        let mut offset = LinearOffsetBuilder::new();
        self.top_tracks.allot(left,&mut offset, &mut secondary);
        self.main.allot(left,&mut offset,&mut secondary);
        self.bottom_tracks.allot(left,&mut offset,&mut secondary);
        /* window etc */
        self.window.allot(0,&mut LinearOffsetBuilder::dud(0),&mut secondary);
        self.window_bottom.allot(0,&mut LinearOffsetBuilder::dud(0),&mut secondary);
        self.window_tracks.allot(0,&mut LinearOffsetBuilder::dud(0),&mut secondary);
        self.window_tracks_bottom.allot(0,&mut LinearOffsetBuilder::dud(0),&mut secondary);
        /* update playing fields */
        self.playingfield = PlayingField::new_height(self.bottom_tracks.max());
        self.playingfield.union(&PlayingField::new_squeeze(left_offset.total_size(),right_offset.total_size()));
    }

    fn playingfield(&self) -> &PlayingField { &self.playingfield }
}

#[derive(Clone)]
pub struct Universe {
    data: Arc<Mutex<UniverseData>>,
    allotment_metadata: AllotmentMetadataStore
}

impl Universe {
    pub fn new(allotment_metadata: &AllotmentMetadataStore) -> Universe {
        Universe {
            data: Arc::new(Mutex::new(UniverseData {
                main: LinearGroup::new(MainTrackRequestCreator(false)),
                top_tracks: LinearGroup::new(MainTrackRequestCreator(false)),
                bottom_tracks: LinearGroup::new(MainTrackRequestCreator(false)),
                left: LinearGroup::new(OffsetAllotmentRequestCreator(CoordinateSystem::SidewaysLeft,false)),
                right: LinearGroup::new(OffsetAllotmentRequestCreator(CoordinateSystem::SidewaysRight,true)),
                window: LinearGroup::new(OffsetAllotmentRequestCreator(CoordinateSystem::Window,false)),
                window_bottom: LinearGroup::new(OffsetAllotmentRequestCreator(CoordinateSystem::WindowBottom,false)),
                window_tracks: LinearGroup::new(OffsetAllotmentRequestCreator(CoordinateSystem::TrackingWindow,false)),
                window_tracks_bottom: LinearGroup::new(OffsetAllotmentRequestCreator(CoordinateSystem::TrackingWindowBottom,false)),
                dustbin: Arc::new(DustbinAllotmentRequest()),
                playingfield: PlayingField::empty()
            })),
            allotment_metadata: allotment_metadata.clone()
        }
    }

    pub fn make_metadata_report(&self) -> AllotmentMetadataReport {
        let mut metadata = vec![];
        lock!(self.data).get_all_metadata(&self.allotment_metadata, &mut metadata);
        AllotmentMetadataReport::new(metadata)
    }

    pub fn make_request(&self, name: &str) -> Option<AllotmentRequest> {
        lock!(self.data).make_request(&self.allotment_metadata,name)
    }

    pub fn union(&mut self, other: &Universe) {
        if Arc::ptr_eq(&self.data,&other.data) { return; }
        let mut self_data = lock!(self.data);
        let other_data = lock!(other.data);
        self_data.union(&other_data);
    }

    pub fn allot(&self) { lock!(self.data).allot(); }
    pub fn playingfield(&self) -> PlayingField { lock!(self.data).playingfield().clone() }
}
