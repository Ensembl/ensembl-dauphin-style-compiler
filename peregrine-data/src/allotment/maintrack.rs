use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, sync::Arc};
use crate::{AllotmentDirection, AllotmentGroup, AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};
use super::{allotment::AllotmentImpl, baseallotmentrequest::BaseAllotmentRequest, lineargroup::{LinearAllotmentImpl, LinearAllotmentRequestCreatorImpl, LinearGroupEntry}};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct OffsetAllotment {
    metadata: AllotmentMetadata,
    direction: AllotmentDirection,
    offset: i64,
    size: i64
}

impl OffsetAllotment {
    pub(crate) fn new(metadata: &AllotmentMetadata, direction: &AllotmentDirection, offset: i64, size: i64) -> OffsetAllotment {
        OffsetAllotment {
            metadata: metadata.clone(),
            direction: direction.clone(),
            offset,size
        }
    }

    fn add_metadata(&self, full_metadata: &mut AllotmentMetadataRequest) {
        full_metadata.add_pair("type","track");
        full_metadata.add_pair("offset",&self.offset.to_string());
        full_metadata.add_pair("height",&self.size.to_string());
    }
}

impl LinearAllotmentImpl for OffsetAllotment {
    fn max(&self) -> i64 { self.offset+self.size }

    fn up(self: Arc<Self>) -> Arc<dyn LinearAllotmentImpl> { self }
}

impl AllotmentImpl for OffsetAllotment {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        let mut output = input.make();
        output.normal += self.offset as f64;
        output
    }

    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        let offset = self.offset as f64;
        values.iter().map(|x| x.map(|y| y+offset)).collect()
    }

    fn direction(&self) -> AllotmentDirection { self.direction.clone() }
}

#[derive(Clone)]
pub struct MainTrackRequest {
    main: Arc<BaseAllotmentRequest<OffsetAllotment>>,
    wallpaper: Arc<BaseAllotmentRequest<OffsetAllotment>>
}

impl MainTrackRequest {
    fn new(metadata: &AllotmentMetadata, group: &AllotmentGroup) -> MainTrackRequest {
        let main = Arc::new(BaseAllotmentRequest::new(metadata,group));
        let wallpaper = Arc::new(BaseAllotmentRequest::new(metadata,group));
        MainTrackRequest {
            main, wallpaper
        }
    }
}

const WALLPAPER : &str = ":wallpaper";

impl LinearGroupEntry for MainTrackRequest {
    fn make(&self, offset: i64, size: i64) {
        self.main.set_allotment(Arc::new(OffsetAllotment::new(&self.main.metadata(),&self.main.direction(),offset,size)));
        self.wallpaper.set_allotment(Arc::new(OffsetAllotment::new(&self.main.metadata(),&self.main.direction(),offset,size)));
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
    fn make(&self, metadata: &AllotmentMetadata, group: &AllotmentGroup) -> Arc<dyn LinearGroupEntry> {
        Arc::new(MainTrackRequest::new(metadata,group))
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
