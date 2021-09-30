use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, sync::Arc};
use crate::{AllotmentDirection, AllotmentGroup, AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};
use super::{baseallotmentrequest::BaseAllotmentRequest, lineargroup::{LinearAllotmentImpl, LinearAllotmentRequestCreatorImpl, LinearGroupEntry}, offsetallotment::OffsetAllotment};

#[derive(Clone)]
pub struct MainTrackRequest {
    main: Arc<BaseAllotmentRequest<OffsetAllotment>>,
    wallpaper: Arc<BaseAllotmentRequest<OffsetAllotment>>
}

impl MainTrackRequest {
    fn new(metadata: &AllotmentMetadata) -> MainTrackRequest {
        let main = Arc::new(BaseAllotmentRequest::new(metadata,&AllotmentGroup::Track));
        let wallpaper = Arc::new(BaseAllotmentRequest::new(metadata,&AllotmentGroup::SpaceLabel(AllotmentDirection::Forward)));
        MainTrackRequest {
            main, wallpaper
        }
    }
}

const WALLPAPER : &str = ":wallpaper";

impl LinearGroupEntry for MainTrackRequest {
    fn make(&self, offset: i64) {
        self.main.set_allotment(Arc::new(OffsetAllotment::new(&self.main.metadata(),&AllotmentGroup::Track,self.main.best_offset(offset),self.main.best_height())));
        self.wallpaper.set_allotment(Arc::new(OffsetAllotment::new(&self.main.metadata(),&AllotmentGroup::SpaceLabel(AllotmentDirection::Forward),self.main.best_offset(offset),64)));
    }

    fn get_all_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let mut full_metadata = AllotmentMetadataRequest::rebuild(&self.main.metadata());
        if let Some(allotment) = self.main.base_allotment() {
            allotment.add_metadata(&mut full_metadata);
        }
        out.push(AllotmentMetadata::new(full_metadata));
        // XXX wallpaper metadata
    }

    fn max(&self) -> i64 { self.main.base_allotment().map(|x| x.max()).unwrap_or(0) }
    fn name(&self) -> &str { self.main.metadata().name() }
    fn priority(&self) -> i64 { self.main.metadata().priority() }

    fn make_request(&self, _allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        if name.ends_with(WALLPAPER) {
            Some(AllotmentRequest::upcast(self.wallpaper.clone()))
        } else {
            Some(AllotmentRequest::upcast(self.main.clone()))
        }
    }
}

pub struct MainTrackRequestCreator();

impl LinearAllotmentRequestCreatorImpl for MainTrackRequestCreator {
    fn make(&self, metadata: &AllotmentMetadata) -> Arc<dyn LinearGroupEntry> {
        Arc::new(MainTrackRequest::new(metadata))
    }

    fn hash(&self, name: &str) -> u64 {
        let prefix = if name.ends_with(WALLPAPER) {
            &name[..name.len()-WALLPAPER.len()]
        } else {
            name
        };
        let mut hasher = DefaultHasher::new();
        prefix.hash(&mut hasher);
        hasher.finish()
    }
}
