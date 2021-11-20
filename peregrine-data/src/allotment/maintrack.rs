use std::{collections::HashMap, hash::{Hash}, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, allotment::allotmentrequest::AllotmentRequestImpl};
use super::{allotment::CoordinateSystem, baseallotmentrequest::{BaseAllotmentRequest, remove_depth, remove_secondary, trim_suffix}, lineargroup::{LinearAllotmentRequestCreatorImpl, LinearGroupEntry, SecondaryPositionStore}, offsetallotment::OffsetAllotment};

/* MainTrack allotments are the allotment spec for the main gb tracks and so have complex spceifiers. The format is
 * track:NAME:(XXX todo sub-tracks) or wallpaper[depth]
 * where [depth] is the drawing priority, a possibly negative number
 */

 #[derive(Clone,PartialEq,Eq,Hash)]
 enum MTVariety {
    Track,
    TrackWindow,
    Wallpaper
}


#[derive(Clone,PartialEq,Eq,Hash)]
struct MTSpecifier {
    name: String,
    variety: MTVariety,
    depth: i8,
    secondary: Option<String>
}

impl MTSpecifier {
    fn new(spec: &str) -> MTSpecifier {
        let mut spec = spec.to_string();
        let depth = remove_depth(&mut spec);
        let secondary = remove_secondary(&mut spec);
        if let Some(main) = trim_suffix("wallpaper",&spec) {
            MTSpecifier { name: main.to_string(), variety: MTVariety::Wallpaper, depth, secondary }
        } else if let Some(main) = trim_suffix("window",&spec) {
            MTSpecifier { name: main.to_string(), variety: MTVariety::TrackWindow, depth, secondary }    
        } else {
            MTSpecifier { name: spec.to_string(), variety: MTVariety::Track, depth, secondary }
        }
    }

    fn sized(&self) -> bool {
        match self.variety {
            MTVariety::Track => true,
            MTVariety::TrackWindow => false,
            MTVariety::Wallpaper => false
        }
    }

    fn name(&self) -> &str { &self.name }
    fn depth(&self) -> i8 { self.depth }

    fn coord_system(&self, reverse: bool) -> CoordinateSystem {
        match (&self.variety,reverse) {
            (MTVariety::Track,_)           => CoordinateSystem::Tracking,
            (MTVariety::TrackWindow,false) => CoordinateSystem::TrackingWindow,
            (MTVariety::TrackWindow,true)  => CoordinateSystem::TrackingWindowBottom,
            (MTVariety::Wallpaper,false)   => CoordinateSystem::Window,
            (MTVariety::Wallpaper,true)    => CoordinateSystem::WindowBottom
        }
    }

    fn get_secondary(&self, default_secondary: i64, secondary_store: &SecondaryPositionStore) -> i64 {
        match self.variety {
            MTVariety::Track => 0,
            MTVariety::TrackWindow => 0,
            MTVariety::Wallpaper => {
                let secondary = self.secondary.as_ref().map(|s| secondary_store.lookup(s)).flatten();
                secondary.map(|p| p.offset).unwrap_or(default_secondary)
            }
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
    fn make(&self, secondary: i64, offset: i64, secondary_store: &SecondaryPositionStore) -> i64 {
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
            let our_secondary = specifier.get_secondary(secondary,secondary_store);
            request.set_allotment(Arc::new(OffsetAllotment::new(&request.coord_system(),request.metadata(),our_secondary,offset,best_offset,best_height,specifier.depth,self.reverse)));
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
            Arc::new(BaseAllotmentRequest::new(&self.metadata,&specifier.coord_system(self.reverse),specifier.depth()))
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
