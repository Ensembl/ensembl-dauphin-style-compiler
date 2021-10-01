use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::{Hash, Hasher}, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{AllotmentDirection, AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, SpaceBasePointRef, shape::shape::FilterMinMax, spacebase::spacebase::SpaceBasePoint};
use super::{allotment::CoordinateSystem, baseallotmentrequest::{BaseAllotmentRequest, remove_depth}, lineargroup::{LinearAllotmentImpl, LinearAllotmentRequestCreatorImpl, LinearGroupEntry}, offsetallotment::OffsetAllotment};

/* MainTrack allotments are the allotment spec for the main gb tracks and so have complex spceifiers. The format is
 * track:NAME:(XXX todo sub-tracks) or wallpaper[depth]
 * where [depth] is the drawing priority, a possibly negative number
 */

 #[derive(Clone,PartialEq,Eq,Hash)]
 enum MTVariety {
    Track,
    Wallpaper
}


#[derive(Clone,PartialEq,Eq,Hash)]
struct MTSpecifier {
    name: String,
    variety: MTVariety,
    depth: i8
}

impl MTSpecifier {
    fn new(spec: &str) -> MTSpecifier {
        let mut spec = spec.to_string();
        let depth = remove_depth(&mut spec);
        let parts = spec.split(":").collect::<Vec<_>>();
        if parts.len() < 2 || parts[0] != "track" {
            MTSpecifier { name: "".to_string(), variety: MTVariety::Track, depth }
        } else if parts.len() > 2 && parts[2] == "wallpaper" {
            MTSpecifier { name: parts[1].to_string(), variety: MTVariety::Wallpaper, depth }
        } else {
            MTSpecifier { name: parts[1].to_string(), variety: MTVariety::Track, depth }
        }
    }

    fn sized(&self) -> bool {
        match self.variety {
            MTVariety::Track => true,
            MTVariety::Wallpaper => false
        }
    }

    fn direction(&self) -> AllotmentDirection { AllotmentDirection::Forward }
    fn depth(&self) -> i8 { self.depth }

    fn coord_system(&self) -> CoordinateSystem {
        match self.variety {
            MTVariety::Track => CoordinateSystem::Track,
            MTVariety::Wallpaper => CoordinateSystem::Space
        }
    }
}

pub struct MainTrackRequest {
    metadata: AllotmentMetadata,
    requests: Mutex<HashMap<MTSpecifier,Arc<BaseAllotmentRequest<OffsetAllotment>>>>
}

impl MainTrackRequest {
    fn new(metadata: &AllotmentMetadata) -> MainTrackRequest {
        MainTrackRequest {
            metadata: metadata.clone(),
            requests: Mutex::new(HashMap::new())
        }
    }
}

impl LinearGroupEntry for MainTrackRequest {
    fn make(&self, offset: i64) -> i64 {
        let mut best_offset = 0;
        let mut best_height = 0;
        let requests = lock!(self.requests);
        for (specifier,request) in requests.iter() {
            if specifier.sized() {
                best_offset = best_offset.max(request.best_offset(offset));
                best_height = best_height.max(request.best_height());
            }
        }
        for (specifier,request) in requests.iter() {
            request.set_allotment(Arc::new(OffsetAllotment::new(request.metadata(),&specifier.direction(),offset,best_offset,best_height,specifier.depth)));
        }
        best_height
    }

    fn get_all_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let requests = lock!(self.requests);
        for (specifier,request) in requests.iter() {
            let mut full_metadata = AllotmentMetadataRequest::rebuild(&self.metadata);
            if specifier.sized() { // XXX wallpaper metadata
                if let Some(allotment) = request.base_allotment() {
                    allotment.add_metadata(&mut full_metadata);
                }
            }
            out.push(AllotmentMetadata::new(full_metadata));
        }
    }

    fn name(&self) -> &str { self.metadata.name() }
    fn priority(&self) -> i64 { self.metadata.priority() }

    fn make_request(&self, _allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let specifier = MTSpecifier::new(name);
        let mut requests = lock!(self.requests);
        let req_impl = requests.entry(specifier.clone()).or_insert_with(|| {
            Arc::new(BaseAllotmentRequest::new(&self.metadata,&specifier.coord_system(),&specifier.direction(),specifier.depth()))
        });
        Some(AllotmentRequest::upcast(req_impl.clone()))
    }
}

pub struct MainTrackRequestCreator();

impl LinearAllotmentRequestCreatorImpl for MainTrackRequestCreator {
    fn make(&self, metadata: &AllotmentMetadata) -> Arc<dyn LinearGroupEntry> {
        Arc::new(MainTrackRequest::new(metadata))
    }

    fn hash(&self, name: &str) -> u64 {
        let specifier = MTSpecifier::new(name);
        let mut hasher = DefaultHasher::new();
        specifier.name.hash(&mut hasher);
        hasher.finish()
    }
}
