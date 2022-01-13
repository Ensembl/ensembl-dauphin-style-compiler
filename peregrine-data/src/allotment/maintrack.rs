use std::{collections::HashMap, hash::{Hash}, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, allotment::allotmentrequest::AllotmentRequestImpl};
use super::{allotment::CoordinateSystem, baseallotmentrequest::{BaseAllotmentRequest}, lineargroup::{lineargroup::{LinearGroupEntry, LinearGroupHelper}, secondary::SecondaryPositionStore}, leafboxallotment::LeafBoxAllotment, basicallotmentspec::BasicAllotmentSpec, treeallotment::{tree_best_offset, tree_best_height}};

/* MainTrack allotments are the allotment spec for the main gb tracks and so have complex spceifiers. The format is
 * track:NAME:(XXX todo sub-tracks) or wallpaper[depth]
 * where [depth] is the drawing priority, a possibly negative number
 */

 fn trim_suffix(suffix: &str, name: &str) -> Option<String> {
    if let Some(start) = name.rfind(":") {
        if &name[start+1..] == suffix {
            return Some(name[0..start].to_string());
        }
    }
    None
}

#[derive(Clone,PartialEq,Eq,Hash)]
enum MTVariety {
    Track,
    TrackWindow,
    Wallpaper
}

impl MTVariety {
    fn from_suffix(spec: &str) -> (MTVariety,String) {
        if let Some(main) = trim_suffix("wallpaper",&spec) {
            (MTVariety::Wallpaper,main)
        } else if let Some(main) = trim_suffix("window",&spec) {
            (MTVariety::TrackWindow,main)
        } else {
            (MTVariety::Track,spec.to_string())
        }
    }
}

#[derive(Clone,PartialEq,Eq,Hash)]
struct MTSpecifier {
    variety: MTVariety,
    base: BasicAllotmentSpec
}

impl MTSpecifier {
    fn new(spec: &str) -> MTSpecifier {
        let base = BasicAllotmentSpec::from_spec(&spec);
        let (variety,main) = MTVariety::from_suffix(&base.name());
        let base = base.with_name(&main);
        MTSpecifier { variety, base }
    }

    fn sized(&self) -> bool {
        match self.variety {
            MTVariety::Track => true,
            MTVariety::TrackWindow => false,
            MTVariety::Wallpaper => false
        }
    }

    fn base(&self) -> &BasicAllotmentSpec { &self.base }

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
                let secondary = self.base.secondary().as_ref().map(|s| secondary_store.lookup(s)).flatten();
                secondary.map(|p| p.offset).unwrap_or(default_secondary)
            }
        }
    }
}

pub struct MainTrackRequest {
    metadata: AllotmentMetadata,
    requests: Mutex<HashMap<MTSpecifier,Arc<BaseAllotmentRequest<LeafBoxAllotment>>>>,
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
    fn allot(&self, secondary: i64, offset: i64, secondary_store: &SecondaryPositionStore) -> i64 {
        let mut best_offset_val = 0;
        let mut best_height_val = 0;
        let requests = lock!(self.requests);
        for (specifier,request) in requests.iter() {
            if specifier.sized() {
                best_offset_val = best_offset_val.max(tree_best_offset(&request,offset));
                best_height_val = best_height_val.max(tree_best_height(&request));
            }
        }
        for (specifier,request) in requests.iter() {
            let our_secondary = specifier.get_secondary(secondary,secondary_store);
            request.set_allotment(Arc::new(LeafBoxAllotment::new(&request.coord_system(),request.metadata(),our_secondary,offset,best_offset_val,best_height_val,specifier.base.depth(),self.reverse)));
        }
        best_height_val
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

    fn name_for_secondary(&self) -> &str { self.metadata.name() }
    fn priority(&self) -> i64 { self.metadata.priority() }

    fn make_request(&self, _allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let specifier = MTSpecifier::new(name);
        let mut requests = lock!(self.requests);
        let req_impl = requests.entry(specifier.clone()).or_insert_with(|| {
            Arc::new(BaseAllotmentRequest::new(&self.metadata,&specifier.coord_system(self.reverse),specifier.base.depth()))
        });
        Some(AllotmentRequest::upcast(req_impl.clone()))
    }
}

pub struct MainTrackRequestCreator(pub bool);

impl LinearGroupHelper for MainTrackRequestCreator {
    type Key = String;

    fn make_linear_group_entry(&self, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry> {
        let specifier = MTSpecifier::new(full_path);
        let name = specifier.base.name();
        let metadata = metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
        Arc::new(MainTrackRequest::new(&metadata,self.0))
    }

    fn entry_key(&self, name: &str) -> String {
        let specifier = MTSpecifier::new(name);
        specifier.base.name().to_string()
    }

    fn is_reverse(&self) -> bool { self.0 }
}
