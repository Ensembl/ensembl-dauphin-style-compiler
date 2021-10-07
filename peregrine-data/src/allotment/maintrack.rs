use std::{collections::HashMap, hash::{Hash, Hasher}, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest};
use super::{allotment::CoordinateSystem, baseallotmentrequest::{BaseAllotmentRequest, remove_depth, trim_suffix}, lineargroup::{ LinearAllotmentRequestCreatorImpl, LinearGroupEntry}, offsetallotment::OffsetAllotment};

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
        if let Some(main) = trim_suffix("wallpaper",&spec) {
            MTSpecifier { name: main.to_string(), variety: MTVariety::Wallpaper, depth }
        } else {
            MTSpecifier { name: spec.to_string(), variety: MTVariety::Track, depth }
        }
    }

    fn sized(&self) -> bool {
        match self.variety {
            MTVariety::Track => true,
            MTVariety::Wallpaper => false
        }
    }

    fn name(&self) -> &str { &self.name }
    fn depth(&self) -> i8 { self.depth }

    fn coord_system(&self) -> CoordinateSystem {
        match self.variety {
            MTVariety::Track => CoordinateSystem::Tracking,
            MTVariety::Wallpaper => CoordinateSystem::Window
        }
    }
}

pub struct MainTrackRequest {
    metadata: AllotmentMetadata,
    requests: Mutex<HashMap<MTSpecifier,Arc<BaseAllotmentRequest<OffsetAllotment>>>>,
    reverse: bool
}

impl MainTrackRequest {
    fn new(metadata: &AllotmentMetadata, reverse: bool) -> MainTrackRequest {
        MainTrackRequest {
            metadata: metadata.clone(),
            requests: Mutex::new(HashMap::new()),
            reverse
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
            request.set_allotment(Arc::new(OffsetAllotment::new(request.metadata(),offset,best_offset,best_height,specifier.depth,self.reverse)));
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
            Arc::new(BaseAllotmentRequest::new(&self.metadata,&specifier.coord_system(),specifier.depth()))
        });
        Some(AllotmentRequest::upcast(req_impl.clone()))
    }
}

pub struct MainTrackRequestCreator(pub bool);

impl LinearAllotmentRequestCreatorImpl for MainTrackRequestCreator {
    fn make(&self, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry> {
        let specifier = MTSpecifier::new(full_path);
        let name = specifier.name();
        let metadata = metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
        Arc::new(MainTrackRequest::new(&metadata,self.0))
    }

    fn base(&self, name: &str) -> String {
        let specifier = MTSpecifier::new(name);
        specifier.name().to_string()
    }

    fn is_reverse(&self) -> bool { self.0 }
}
