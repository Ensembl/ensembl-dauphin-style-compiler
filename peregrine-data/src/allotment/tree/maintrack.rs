use std::sync::{Arc, Mutex};
use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, allotment::{lineargroup::{lineargroup::{LinearGroupEntry, LinearGroupHelper, LinearGroup}}, core::{arbitrator::{Arbitrator, SymbolicAxis}}, tree::maintrackspec::MTSection}, CoordinateSystem};

use super::{leaftransformer::{LeafGeometry}, allotmentbox::{AllotmentBox, AllotmentBoxBuilder}, maintrackspec::MTSpecifier, collidegroup::CollideGroupLinearHelper, leafboxlinearentry::BoxAllotmentLinearGroupHelper};

/* MainTrack allotments are the allotment spec for the main gb tracks and so have complex spceifiers. The format is
 * track:NAME:(XXX todo sub-tracks) or wallpaper[depth]
 * where [depth] is the drawing priority, a possibly negative number
 */

identitynumber!(IDS3);

pub struct MainTrackRequest {
    metadata: AllotmentMetadata,
    header: Mutex<LinearGroup<BoxAllotmentLinearGroupHelper>>,
    requests: Mutex<LinearGroup<CollideGroupLinearHelper>>,
    id: u64
}

impl MainTrackRequest {
    fn new(metadata: &AllotmentMetadata, geometry: &LeafGeometry) -> MainTrackRequest {
        let window_geometry = geometry.with_new_coord_system(&CoordinateSystem::Window);
        MainTrackRequest {
            metadata: metadata.clone(),
            header: Mutex::new(LinearGroup::new(&window_geometry,BoxAllotmentLinearGroupHelper)),
            requests: Mutex::new(LinearGroup::new(geometry,CollideGroupLinearHelper)),
            id: IDS3.next()
        }
    }
}

impl LinearGroupEntry for MainTrackRequest {
    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let mut new = AllotmentMetadataRequest::rebuild(&self.metadata);
        for (_,entry) in lock!(self.requests).iter() {
            entry.add_allotment_metadata_values(&mut new);
        }
        out.push(AllotmentMetadata::new(new));
    }

    fn priority(&self) -> i64 { self.metadata.priority() }

    fn make_request(&self, geometry: &LeafGeometry, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let spec = MTSpecifier::new(name);
        match spec.section() {
            MTSection::Main => {
                lock!(self.requests).make_request(allotment_metadata,name)
            },
            MTSection::Header => {
                lock!(self.header).make_request(allotment_metadata,name)
            }
        }
    }

    fn bump(&self, arbitrator: &mut Arbitrator) {
        use web_sys::console;
        console::log_1(&format!("MaintrackRequestEntry bump {}",self.id).into());
        lock!(self.requests).bump(arbitrator);
    }

    fn allot(&self, arbitrator: &mut Arbitrator) -> AllotmentBox {
        let mut main_builder = AllotmentBoxBuilder::empty(0);

        /* the top of the track, where the title goes */
        let mut header_builder = AllotmentBoxBuilder::empty(0);
        header_builder.append_all(lock!(self.header).allot(arbitrator));
        let header = AllotmentBox::new(header_builder);
        main_builder.append(header);

        /* the main bit of a track, where the data all goes */
        let mut data_builder = AllotmentBoxBuilder::new(&self.metadata,0);
        data_builder.overlay_all(lock!(self.requests).allot(arbitrator));
        let data_box = AllotmentBox::new(data_builder);
        main_builder.append(data_box);

        /* make it so! */
        let main_box = AllotmentBox::new(main_builder);
        arbitrator.add_symbolic(&SymbolicAxis::ScreenVert, self.metadata.name(), main_box.top_delayed());
        main_box
    }
}

use lazy_static::lazy_static;
use identitynumber::identitynumber;

pub struct MainTrackLinearHelper(u64);
identitynumber!(IDS);

impl MainTrackLinearHelper {
    pub fn new() -> MainTrackLinearHelper {
        MainTrackLinearHelper(IDS.next())
    }
}

impl LinearGroupHelper for MainTrackLinearHelper {
    type Key = String;
    type Value = MainTrackRequest;

    fn make_linear_group_entry(&self, geometry: &LeafGeometry, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<MainTrackRequest> {
        use web_sys::console;
        let specifier = MTSpecifier::new(full_path);
        let name = specifier.base().name();
        let metadata = metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
        let out = Arc::new(MainTrackRequest::new(&metadata,geometry));
        console::log_1(&format!("make_linear_group_entry for {} returning {}",self.0,out.id).into());
        out
    }

    fn entry_key(&self, name: &str) -> String {
        let specifier = MTSpecifier::new(name);
        specifier.base().name().to_string()
    }

    fn bump(&self, arbitrator: &mut Arbitrator) {
        use web_sys::console;
        console::log_1(&format!("MainTrackLinearHelper bumping {}",self.0).into());
    }
}
