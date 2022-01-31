use std::sync::{Arc, Mutex};

use crate::allotment::lineargroup::lineargroup::LinearGroup;
use crate::allotment::tree::allotmentbox::{AllotmentBoxBuilder, AllotmentBox};
use crate::allotment::tree::leafboxlinearentry::BoxAllotmentLinearGroupHelper;
use crate::allotment::tree::leaftransformer::LeafGeometry;
use crate::allotment::tree::maintrack::MainTrackLinearHelper;
use crate::api::PlayingField;
use crate::core::pixelsize::PixelSize;
use crate::{AllotmentMetadata, AllotmentMetadataReport, AllotmentMetadataStore, AllotmentRequest, CoordinateSystem, Scale};
use peregrine_toolkit::lock;

use super::allotmentrequest::AllotmentRequestImpl;
use super::arbitrator::Arbitrator;
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

impl UniverseData {
    fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        if name == "" {
            Some(AllotmentRequest::upcast(self.dustbin.clone()))
        } else {
            if let Some(start) = name.find(":") {
                let prefix = &name[0..start];
                match prefix {
                    "track" => { return self.main.make_request(allotment_metadata,&name); },
                    "track-top" => { return self.top_tracks.make_request(allotment_metadata,&name); },
                    "track-bottom" => { return self.bottom_tracks.make_request(allotment_metadata,&name); },
                    "track-window" => { return self.window_tracks.make_request(allotment_metadata,&name); },
                    "window" => { return self.window.make_request(allotment_metadata,&name); },
                    "window-bottom" => { return self.window_bottom.make_request(allotment_metadata,&name); },
                    "track-window-bottom" => { return self.window_tracks_bottom.make_request(allotment_metadata,&name); },
                    "left" => { return self.left.make_request(allotment_metadata,&name); },
                    "right" => { return self.right.make_request(allotment_metadata,&name); },
                    _ => {} 
                }
            }
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

    fn real_calc_max_px_per_bp(&self, scale: &Scale, pixel_size: &PixelSize) -> f64 {
        let bp_per_carriage = scale.bp_in_carriage() as f64;
        let max_px_per_carriage = pixel_size.max_px_per_carriage() as f64;
        max_px_per_carriage / bp_per_carriage
    }

    fn calc_max_px_per_bp(&self, scale: Option<&Scale>, pixel_size: Option<&PixelSize>) -> Option<f64> {
        if let (Some(scale),Some(pixel_size)) = (scale,pixel_size) {
            Some(self.real_calc_max_px_per_bp(scale,pixel_size))
        } else {
            None
        }
    }

    fn allot(&mut self, scale: Option<&Scale>, pixel_size: Option<&PixelSize>) {
        let max_px_per_bp = self.calc_max_px_per_bp(scale,pixel_size);
        let mut arbitrator = Arbitrator::new(max_px_per_bp);

        /*
         * LEFT & RIGHT
         */

        /* left */
        let mut left_offset_builder = AllotmentBoxBuilder::empty(0);
        left_offset_builder.append_all(self.left.allot(&mut arbitrator));
        let left_offset = AllotmentBox::new(left_offset_builder);
        left_offset.set_root(0,0);

        /* right */
        let mut right_offset_builder = AllotmentBoxBuilder::empty(0);
        right_offset_builder.append_all(self.right.allot(&mut arbitrator));
        let right_offset = AllotmentBox::new(right_offset_builder);
        right_offset.set_root(0,0);

        let left = left_offset.total_height();

        /*
         * MAIN
         */

        /* main top */
        let mut top_offset_builder = AllotmentBoxBuilder::empty(0);
        top_offset_builder.append_all(self.top_tracks.allot(&mut arbitrator));
        top_offset_builder.append_all(self.main.allot(&mut arbitrator));
        let top_offset = AllotmentBox::new(top_offset_builder);
        top_offset.set_root(0,left);

        /* main bottom */
        let mut bottom_offset_builder = AllotmentBoxBuilder::empty(0);
        bottom_offset_builder.append_all(self.bottom_tracks.allot(&mut arbitrator));
        let bottom_offset = AllotmentBox::new(bottom_offset_builder);
        bottom_offset.set_root(0,left);

        /*
         * WINDOW
         */
        
        let mut window_builder = AllotmentBoxBuilder::empty(0);
        let mut window_tracks_builder = AllotmentBoxBuilder::empty(0);
        
        window_builder.overlay_all(self.window.allot(&mut arbitrator));
        window_builder.overlay_all(self.window_bottom.allot(&mut arbitrator));
        window_tracks_builder.overlay_all(self.window_tracks.allot(&mut arbitrator));
        window_tracks_builder.overlay_all(self.window_tracks_bottom.allot(&mut arbitrator));
        let window = AllotmentBox::new(window_builder);
        let window_tracks = AllotmentBox::new(window_tracks_builder);
        window.set_root(0,0);
        window_tracks.set_root(0,left);

        /* update playing fields */
        self.playingfield = PlayingField::new_height(top_offset.total_height()+bottom_offset.total_height());
        self.playingfield.union(&PlayingField::new_squeeze(left_offset.total_height(),right_offset.total_height()));
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
        let right_geometry = LeafGeometry::new(CoordinateSystem::SidewaysRight,true);
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

    pub fn allot(&self, scale: Option<&Scale>, pixel_size: Option<&PixelSize>) { lock!(self.data).allot(scale,pixel_size); }

    pub fn playingfield(&self) -> PlayingField { lock!(self.data).playingfield().clone() }
}
