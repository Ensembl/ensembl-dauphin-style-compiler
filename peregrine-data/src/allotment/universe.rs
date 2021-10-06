use std::sync::{Arc, Mutex};

use crate::{AllotmentMetadata, AllotmentMetadataReport, AllotmentMetadataStore, AllotmentRequest, CoordinateSystem};
use peregrine_toolkit::lock;

use super::{dustbinallotment::DustbinAllotmentRequest,  maintrack::MainTrackRequestCreator, offsetallotment::OffsetAllotmentRequestCreator};
use super::lineargroup::LinearRequestGroup;

struct UniverseData {
    dustbin: Arc<DustbinAllotmentRequest>,
    main: LinearRequestGroup<MainTrackRequestCreator>,
    top_tracks: LinearRequestGroup<OffsetAllotmentRequestCreator>,
    bottom_tracks: LinearRequestGroup<OffsetAllotmentRequestCreator>,
    window: LinearRequestGroup<OffsetAllotmentRequestCreator>,
}

impl UniverseData {
    fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        if name == "" {
            Some(AllotmentRequest::upcast(self.dustbin.clone()))
        } else if name.starts_with("track:") {
            self.main.make_request(allotment_metadata,name)
        } else if name.starts_with("window:") {
            self.window.make_request(allotment_metadata,name)
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
        self.main.allot();
        self.top_tracks.allot();
        self.bottom_tracks.allot();
        self.window.allot();
    }

    pub fn height(&self) -> i64 {
        self.top_tracks.max() + self.main.max() + self.bottom_tracks.max()
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
                main: LinearRequestGroup::new(MainTrackRequestCreator()),
                top_tracks: LinearRequestGroup::new(OffsetAllotmentRequestCreator(CoordinateSystem::Tracking)),
                bottom_tracks: LinearRequestGroup::new(OffsetAllotmentRequestCreator(CoordinateSystem::Tracking)),
                window: LinearRequestGroup::new(OffsetAllotmentRequestCreator(CoordinateSystem::Window)),
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
