use std::sync::{Arc, Mutex};

use crate::allotment::lineargroup::lineargroup::LinearGroup;
use crate::allotment::lineargroup::offsetbuilder::LinearOffsetBuilder;
use crate::allotment::lineargroup::secondary::{SecondaryPositionResolver};
use crate::allotment::tree::leafboxlinearentry::BoxAllotmentLinearGroupHelper;
use crate::allotment::tree::leafboxtransformer::LeafGeometry;
use crate::allotment::tree::maintrack::MainTrackLinearHelper;
use crate::api::PlayingField;
use crate::{AllotmentMetadata, AllotmentMetadataReport, AllotmentMetadataStore, AllotmentRequest, CoordinateSystem};
use peregrine_toolkit::lock;

use super::allotmentrequest::AllotmentRequestImpl;
use super::dustbinallotment::DustbinAllotment;

struct UniverseData {
    dustbin: Arc<AllotmentRequestImpl<DustbinAllotment>>,
    main: LinearGroup<MainTrackLinearHelper>,
    top_tracks: LinearGroup<MainTrackLinearHelper>,
    bottom_tracks: LinearGroup<MainTrackLinearHelper>,
    window_tracks: LinearGroup<BoxAllotmentLinearGroupHelper>,
    window_tracks_bottom: LinearGroup<BoxAllotmentLinearGroupHelper>,
    left: LinearGroup<BoxAllotmentLinearGroupHelper>,
    right: LinearGroup<BoxAllotmentLinearGroupHelper>,
    window: LinearGroup<BoxAllotmentLinearGroupHelper>,
    window_bottom: LinearGroup<BoxAllotmentLinearGroupHelper>,
    playingfield: PlayingField
}

pub(super) fn trim_prefix(prefix: &str, name: &str) -> Option<String> {
    if let Some(start) = name.find(":") {
        if &name[0..start] == prefix {
            return Some(name[start+1..].to_string());
        }
    }
    None
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
    }

    fn allot(&mut self) {
        /* Left and Right */
        let mut secondary = SecondaryPositionResolver::new();
        let mut left_offset = LinearOffsetBuilder::new();
        self.left.allot(&None,&mut left_offset,&mut secondary);
        let mut right_offset = LinearOffsetBuilder::new();
        self.right.allot(&None,&mut right_offset, &mut secondary);
        let left = left_offset.size();
        /* Main run */
        let mut top_offset = LinearOffsetBuilder::new();
        let mut bottom_offset = LinearOffsetBuilder::new();
        self.top_tracks.allot(&Some(left),&mut top_offset, &mut secondary);
        self.main.allot(&Some(left),&mut top_offset,&mut secondary);
        self.bottom_tracks.allot(&Some(left),&mut bottom_offset,&mut secondary);
        /* window etc */
        self.window.allot(&None,&mut LinearOffsetBuilder::dud(0),&mut secondary);
        self.window_bottom.allot(&None,&mut LinearOffsetBuilder::dud(0),&mut secondary);
        self.window_tracks.allot(&None,&mut LinearOffsetBuilder::dud(0),&mut secondary);
        self.window_tracks_bottom.allot(&None,&mut LinearOffsetBuilder::dud(0),&mut secondary);
        /* update playing fields */
        self.playingfield = PlayingField::new_height(top_offset.size()+bottom_offset.size());
        self.playingfield.union(&PlayingField::new_squeeze(left_offset.size(),right_offset.size()));
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
        let main_geometry = LeafGeometry::new(CoordinateSystem::Tracking,false);
        let left_geometry = LeafGeometry::new(CoordinateSystem::SidewaysLeft,false);
        let right_geometry = LeafGeometry::new(CoordinateSystem::SidewaysLeft,true);
        let window_geometry = LeafGeometry::new(CoordinateSystem::Window,false);
        let window_bottom_geometry = LeafGeometry::new(CoordinateSystem::WindowBottom,false);
        let wintrack_geometry = LeafGeometry::new(CoordinateSystem::TrackingWindow,false);
        let wintrack_bottom_geometry = LeafGeometry::new(CoordinateSystem::TrackingWindowBottom,false);
        Universe {
            data: Arc::new(Mutex::new(UniverseData {
                main: LinearGroup::new(&main_geometry,MainTrackLinearHelper),
                top_tracks: LinearGroup::new(&main_geometry,MainTrackLinearHelper),
                bottom_tracks: LinearGroup::new(&main_geometry,MainTrackLinearHelper),
                left: LinearGroup::new(&left_geometry,BoxAllotmentLinearGroupHelper),
                right: LinearGroup::new(&right_geometry,BoxAllotmentLinearGroupHelper),
                window: LinearGroup::new(&window_geometry,BoxAllotmentLinearGroupHelper),
                window_bottom: LinearGroup::new(&window_bottom_geometry,BoxAllotmentLinearGroupHelper),
                window_tracks: LinearGroup::new(&wintrack_geometry,BoxAllotmentLinearGroupHelper),
                window_tracks_bottom: LinearGroup::new(&wintrack_bottom_geometry,BoxAllotmentLinearGroupHelper),
                dustbin: Arc::new(AllotmentRequestImpl::new_dustbin()),
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
