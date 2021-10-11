use std::sync::{Arc, Mutex};

use crate::{AllotmentMetadata, AllotmentMetadataReport, AllotmentMetadataStore, AllotmentRequest, CoordinateSystem};
use peregrine_toolkit::lock;

use super::baseallotmentrequest::trim_prefix;
use super::{dustbinallotment::DustbinAllotmentRequest,  maintrack::MainTrackRequestCreator, offsetallotment::OffsetAllotmentRequestCreator};
use super::lineargroup::{LinearOffsetBuilder, LinearRequestGroup};

struct UniverseData {
    dustbin: Arc<DustbinAllotmentRequest>,
    main: LinearRequestGroup<MainTrackRequestCreator>,
    top_tracks: LinearRequestGroup<MainTrackRequestCreator>,
    bottom_tracks: LinearRequestGroup<MainTrackRequestCreator>,
    window: LinearRequestGroup<OffsetAllotmentRequestCreator>,
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
        } else if let Some(suffix) = trim_prefix("window",name) {
            self.window.make_request(allotment_metadata,&suffix,&name)
        } else {
            None
        }
    }

    fn union(&mut self, other: &UniverseData) {
        self.top_tracks.union(&other.top_tracks);
        self.bottom_tracks.union(&other.bottom_tracks);
        self.main.union(&other.main);
        self.window.union(&other.window);
    }

    fn get_all_metadata(&self,allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        self.main.get_all_metadata(allotment_metadata,out);
        self.top_tracks.get_all_metadata(allotment_metadata,out);
        self.bottom_tracks.get_all_metadata(allotment_metadata,out);
        self.window.get_all_metadata(allotment_metadata,out);
    }

    fn allot(&mut self) {
        let mut offset = LinearOffsetBuilder::new();
        self.top_tracks.allot(&mut offset);
        self.main.allot(&mut offset);
        self.bottom_tracks.allot(&mut offset);
        self.window.allot(&mut LinearOffsetBuilder::new());
    }

    pub fn height(&self) -> i64 {
        self.bottom_tracks.max()
    }
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
                main: LinearRequestGroup::new(MainTrackRequestCreator(false)),
                top_tracks: LinearRequestGroup::new(MainTrackRequestCreator(false)),
                bottom_tracks: LinearRequestGroup::new(MainTrackRequestCreator(true)),
                window: LinearRequestGroup::new(OffsetAllotmentRequestCreator(CoordinateSystem::Window,false)),
                dustbin: Arc::new(DustbinAllotmentRequest())
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

    pub fn allot(&self) {
        lock!(self.data).allot();
    }

    pub fn height(&self) -> i64 {
        lock!(self.data).height()
    }
}
