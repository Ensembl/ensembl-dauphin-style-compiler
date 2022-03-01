use std::borrow::Borrow;
use std::sync::{Arc, Mutex};

use crate::allotment::lineargroup::lineargroup::LinearGroup;
use crate::allotment::tree::allotmentbox::{AllotmentBoxBuilder, AllotmentBox};
use crate::allotment::tree::leafboxlinearentry::BoxAllotmentLinearGroupHelper;
use crate::allotment::tree::maintrack::MainTrackLinearHelper;
use crate::api::PlayingField;
use crate::{AllotmentMetadata, AllotmentMetadataReport, AllotmentMetadataStore, AllotmentRequest, CoordinateSystem, CarriageExtent, CoordinateSystemVariety};
use peregrine_toolkit::lock;
use peregrine_toolkit::puzzle::{PuzzleSolution, Puzzle, PuzzleBuilder};

use super::allotmentrequest::AllotmentRequestImpl;
use super::arbitrator::Arbitrator;

struct CarriageUniverseData {
    puzzle: PuzzleBuilder,
    dustbin: Arc<AllotmentRequestImpl>,
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

impl CarriageUniverseData {
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

    fn union(&mut self, other: &CarriageUniverseData) {
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

    fn get_all_metadata(&self, solution: &PuzzleSolution, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        self.main.get_all_metadata(solution,allotment_metadata,out);
    }

    fn allot(&mut self, extent: Option<&CarriageExtent>) -> PuzzleSolution {
        let mut arbitrator = Arbitrator::new(extent,&self.puzzle);
        let puzzle = Puzzle::new(arbitrator.puzzle().clone());
        let mut solution = PuzzleSolution::new(&puzzle);

        self.left.bump(&mut arbitrator);
        self.right.bump(&mut arbitrator);
        self.main.bump(&mut arbitrator);
        self.top_tracks.bump(&mut arbitrator);
        self.bottom_tracks.bump(&mut arbitrator);
        self.window.bump(&mut arbitrator);
        self.window_bottom.bump(&mut arbitrator);
        self.window_tracks.bump(&mut arbitrator);
        self.window_tracks_bottom.bump(&mut arbitrator);

        /*
         * LEFT & RIGHT
         */

        /* left */
        let mut left_offset_builder = AllotmentBoxBuilder::empty(&arbitrator,0.,&None);
        left_offset_builder.append_all(self.left.allot(&mut arbitrator));
        let left_offset = AllotmentBox::new(left_offset_builder);
        left_offset.set_root(&mut solution, 0.,0.);

        /* right */
        let mut right_offset_builder = AllotmentBoxBuilder::empty(&arbitrator,0.,&None);
        right_offset_builder.append_all(self.right.allot(&mut arbitrator));
        let right_offset = AllotmentBox::new(right_offset_builder);
        right_offset.set_root(&mut solution, 0.,0.);

        let left = left_offset.total_height(&mut solution);

        /*
         * MAIN
         */

        /* main top */
        let mut top_offset_builder = AllotmentBoxBuilder::empty(&arbitrator,0.,&None);
        top_offset_builder.append_all(self.top_tracks.allot(&mut arbitrator));
        top_offset_builder.append_all(self.main.allot(&mut arbitrator));
        let top_offset = AllotmentBox::new(top_offset_builder);
        top_offset.set_root(&mut solution,0.,left as f64);

        /* main bottom */
        let mut bottom_offset_builder = AllotmentBoxBuilder::empty(&arbitrator,0.,&None);
        bottom_offset_builder.append_all(self.bottom_tracks.allot(&mut arbitrator));
        let bottom_offset = AllotmentBox::new(bottom_offset_builder);
        bottom_offset.set_root(&mut solution,0.,left as f64);

        /*
         * WINDOW
         */
        
        let mut window_builder = AllotmentBoxBuilder::empty(&arbitrator,0.,&None);
        let mut window_tracks_builder = AllotmentBoxBuilder::empty(&arbitrator,0.,&None);
        
        window_builder.overlay_all(self.window.allot(&mut arbitrator));
        window_builder.overlay_all(self.window_bottom.allot(&mut arbitrator));
        window_tracks_builder.overlay_all(self.window_tracks.allot(&mut arbitrator));
        window_tracks_builder.overlay_all(self.window_tracks_bottom.allot(&mut arbitrator));
        let window = AllotmentBox::new(window_builder);
        let window_tracks = AllotmentBox::new(window_tracks_builder);
        window.set_root(&mut solution, 0.,0.);
        window_tracks.set_root(&mut solution,0.,left as f64);

        /* update playing fields */
        self.playingfield = PlayingField::new_height((top_offset.total_height(&solution)+bottom_offset.total_height(&solution)) as i64);
        self.playingfield.union(&PlayingField::new_squeeze(left_offset.total_height(&solution) as i64,right_offset.total_height(&solution) as i64));

        solution
    }

    fn playingfield(&self) -> &PlayingField { &self.playingfield }
}

#[derive(Clone)]
pub struct CarriageUniverse {
    solution: Arc<Mutex<Option<PuzzleSolution>>>, // XXX
    data: Arc<Mutex<CarriageUniverseData>>,
    allotment_metadata: AllotmentMetadataStore
}

impl CarriageUniverse {
    pub fn new(allotment_metadata: &AllotmentMetadataStore) -> CarriageUniverse {
        CarriageUniverse {
            data: Arc::new(Mutex::new(CarriageUniverseData {
                puzzle: PuzzleBuilder::new(),
                main: LinearGroup::new(&CoordinateSystem(CoordinateSystemVariety::Tracking,false),MainTrackLinearHelper),
                top_tracks: LinearGroup::new(&CoordinateSystem(CoordinateSystemVariety::Tracking,false),MainTrackLinearHelper),
                bottom_tracks: LinearGroup::new(&CoordinateSystem(CoordinateSystemVariety::Tracking,true),MainTrackLinearHelper),
                left: LinearGroup::new(&CoordinateSystem(CoordinateSystemVariety::Sideways,false),BoxAllotmentLinearGroupHelper),
                right: LinearGroup::new(&CoordinateSystem(CoordinateSystemVariety::Sideways,true),BoxAllotmentLinearGroupHelper),
                window: LinearGroup::new(&CoordinateSystem(CoordinateSystemVariety::Window,false),BoxAllotmentLinearGroupHelper),
                window_bottom: LinearGroup::new(&CoordinateSystem(CoordinateSystemVariety::Window,true),BoxAllotmentLinearGroupHelper),
                window_tracks: LinearGroup::new(&CoordinateSystem(CoordinateSystemVariety::TrackingWindow,false),BoxAllotmentLinearGroupHelper),
                window_tracks_bottom: LinearGroup::new(&CoordinateSystem(CoordinateSystemVariety::TrackingWindow,true),BoxAllotmentLinearGroupHelper),
                dustbin: Arc::new(AllotmentRequestImpl::new_dustbin()),
                playingfield: PlayingField::empty()
            })),
            allotment_metadata: allotment_metadata.clone(),
            solution: Arc::new(Mutex::new(None))
        }
    }

    pub fn puzzle(&self) -> PuzzleBuilder { lock!(self.data).puzzle.clone() }

    pub fn make_metadata_report(&self) -> AllotmentMetadataReport {
        let mut metadata = vec![];
        if let Some(solution) = lock!(self.solution).as_ref() {
            lock!(self.data).get_all_metadata(solution,&self.allotment_metadata, &mut metadata);
        }
        AllotmentMetadataReport::new(metadata)
    }

    pub fn make_request(&self, name: &str) -> Option<AllotmentRequest> {
        lock!(self.data).make_request(&self.allotment_metadata,name)
    }

    pub fn union(&mut self, other: &CarriageUniverse) {
        if Arc::ptr_eq(&self.data,&other.data) { return; }
        let mut self_data = lock!(self.data);
        let other_data = lock!(other.data);
        self_data.union(&other_data);
    }

    pub fn allot(&self, extent: Option<&CarriageExtent>) { 
        let solution = lock!(self.data).allot(extent);
        *lock!(self.solution) = Some(solution);
    }

    pub fn playingfield(&self) -> PlayingField { lock!(self.data).playingfield().clone() }
}
