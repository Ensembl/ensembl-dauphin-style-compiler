use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzleBuilder, PuzzlePiece, DerivedPuzzlePiece, ClonablePuzzleValue, PuzzleValue, ConstantPuzzlePiece, FoldValue, DelayedPuzzleValue}, lock, log};

use crate::{allotment::{core::{allotmentmetadata::{AllotmentMetadataBuilder, AllotmentMetadataGroup}, aligner::Aligner, heighttracker::HeightTrackerPieces, carriageuniverse::CarriageUniversePrep}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentName, AllotmentNamePart}}, boxes::boxtraits::Stackable, util::rangeused::RangeUsed}, CoordinateSystem};

use super::{boxtraits::{Coordinated, StackableAddable}};

fn draw_top(top: &DelayedPuzzleValue<f64>, padding_top: f64) -> PuzzleValueHolder<f64> {
    PuzzleValueHolder::new(DerivedPuzzlePiece::new(top.clone(),move |top| *top + padding_top))
}

fn height(puzzle: &PuzzleBuilder, child_height: &PuzzlePiece<f64>, tracked_height: &DelayedPuzzleValue<f64>, min_height: f64, padding_top: f64, padding_bottom: f64) -> PuzzleValueHolder<f64> {
    let mut piece = puzzle.new_piece();
    #[cfg(debug_assertions)]
    piece.set_name("padder/height");
    let child_height = child_height.clone();
    let tracked_height = tracked_height.clone();
    piece.add_solver(&[child_height.dependency(),tracked_height.dependency()], move |solution| {
        let tracked_height = tracked_height.get_clone(solution);
        let internal_height = child_height.get_clone(solution).max(min_height);
        let external_height = padding_top + internal_height + padding_bottom;
        Some(external_height.max(tracked_height))
    });
    PuzzleValueHolder::new(piece)
}

pub struct Padder<T> {
    builder: PuzzleBuilder,
    child: Box<T>,
    coord_system: CoordinateSystem,
    /* incoming variables */
    top: DelayedPuzzleValue<f64>,
    /* outgoing variables */
    info: PadderInfo,
    height: PuzzleValueHolder<f64>,
    ranges: Arc<Mutex<FoldValue<RangeUsed<f64>>>>,
    full_range: PuzzleValueHolder<RangeUsed<f64>>
}

#[derive(Clone)]
pub struct PadderInfo {
    pub child_height: PuzzlePiece<f64>, /* children set this */
    pub draw_top: PuzzleValueHolder<f64>
}

impl<T: Clone> Clone for Padder<T> {
    fn clone(&self) -> Self {
        Self {
            builder: self.builder.clone(),
            child: self.child.clone(),
            ranges: self.ranges.clone(),
            coord_system: self.coord_system.clone(),
            top: self.top.clone(),
            info: self.info.clone(),
            height: self.height.clone(),
            full_range: self.full_range.clone()
        }
    }
}

fn add_report(metadata: &mut AllotmentMetadataBuilder, in_values: &HashMap<String,String>, top: &PuzzleValueHolder<f64>, height: &PuzzleValueHolder<f64>) {
    let mut values = HashMap::new();
    for (key,value) in in_values {
        values.insert(key.to_string(),PuzzleValueHolder::new(ConstantPuzzlePiece::new(value.to_string())));
    }
    values.insert("offset".to_string(), PuzzleValueHolder::new(DerivedPuzzlePiece::new(top.clone(),|v| v.to_string())));
    values.insert("height".to_string(), PuzzleValueHolder::new(DerivedPuzzlePiece::new(height.clone(),|v| v.to_string())));
    let group = AllotmentMetadataGroup::new(values);
    metadata.add(group);
}

impl<T> Padder<T> {
    pub(crate) fn new<F>(prep: &mut CarriageUniversePrep, name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner, ctor: F) -> Padder<T> where F: FnOnce(&mut CarriageUniversePrep, &PadderInfo) -> T {
        let top = DelayedPuzzleValue::new(&prep.puzzle);
        let padding_top = style.padding.padding_top;
        let padding_bottom = style.padding.padding_bottom;
        let min_height = style.padding.min_height;
        let draw_top = draw_top(&top,padding_top);
        let mut child_height = prep.puzzle.new_piece();
        #[cfg(debug_assertions)]
        child_height.set_name("padder/child-height");
        let tracked_height = DelayedPuzzleValue::new(&prep.puzzle);
        let height = height(&prep.puzzle,&child_height,&tracked_height,min_height,padding_top,padding_bottom);
        let info = PadderInfo {
            draw_top, child_height
        };
        if let Some(report) = &style.padding.report {
            add_report(&mut prep.metadata,report,&PuzzleValueHolder::new(top.clone()),&height);
        }
        let child = ctor(prep,&info);
        let mut full_range = prep.puzzle.new_piece();
        #[cfg(debug_assertions)]
        full_range.set_name("padder/full_range");
        let ranges = Arc::new(Mutex::new(FoldValue::new(full_range.clone(), |x : RangeUsed<f64>,y| x.merge(&y))));
        let ranges2 = ranges.clone();
        prep.puzzle.add_ready(move |_| lock!(ranges2).build());
        if let Some(datum) = &style.set_align {
            aligner.set_datum(&prep.puzzle,datum,&PuzzleValueHolder::new(top.clone()));
        }
        if style.tracked_height {
            let our_name = AllotmentName::from_part(name);
            prep.height_tracker.add(&our_name,&height);
            let global_piece = prep.height_tracker.get_piece(&our_name).clone();
            tracked_height.set(&mut prep.puzzle,PuzzleValueHolder::new(global_piece));
        } else {
            tracked_height.set(&mut prep.puzzle,PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)));
        }
        Padder {
            builder: prep.puzzle.clone(),
            child: Box::new(child),
            ranges,
            coord_system: style.coord_system.clone(),
            top, height,
            info, 
            full_range: PuzzleValueHolder::new(full_range.clone())
        }
    }
}

impl<T> Coordinated for Padder<T> {
    fn coordinate_system(&self) -> &CoordinateSystem { &self.coord_system }
}

impl<T> Stackable for Padder<T> {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.top.set(&self.builder,value);
    }

    fn height(&self) -> PuzzleValueHolder<f64> { self.height.clone() }

    fn top_anchor(&self, _puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> {
        PuzzleValueHolder::new(self.info.draw_top.clone())
    }

    fn full_range(&self) -> PuzzleValueHolder<RangeUsed<f64>> { self.full_range.clone() }
}

impl<T: StackableAddable> StackableAddable for Padder<T> {
    fn add_child(&mut self, child: &dyn Stackable, priority: i64) {
        self.child.add_child(child,priority);
        lock!(self.ranges).add(&child.full_range());
    }
}
