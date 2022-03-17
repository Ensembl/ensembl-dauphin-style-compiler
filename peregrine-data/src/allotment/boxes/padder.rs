use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzleBuilder, PuzzlePiece, DerivedPuzzlePiece, ClonablePuzzleValue, PuzzleValue, ConstantPuzzlePiece}, log, lock};

use crate::{allotment::{core::{arbitrator::Arbitrator, allotmentmetadata2::{AllotmentMetadata2Builder, AllotmentMetadataGroup}, rangeused::RangeUsed}, style::{style::Padding}, boxes::boxtraits::Stackable}, AllotmentMetadata, CoordinateSystem};

use super::{boxtraits::{Coordinated, StackableAddable}, rangecontainer::RangeMerger};

fn draw_top(top: &PuzzlePiece<f64>, padding_top: f64) -> PuzzleValueHolder<f64> {
    PuzzleValueHolder::new(DerivedPuzzlePiece::new(top.clone(),move |top| *top + padding_top))
}

fn height(puzzle: &PuzzleBuilder, child_height: &PuzzlePiece<f64>, min_height: f64, padding_top: f64, padding_bottom: f64) -> PuzzleValueHolder<f64> {
    let mut piece = puzzle.new_piece();
    #[cfg(debug_assertions)]
    piece.set_name("padder/height");
    let child_height = child_height.clone();
    piece.add_solver(&[child_height.dependency()], move |solution| {
        let internal_height = child_height.get_clone(solution).max(min_height);
        Some(padding_top + internal_height + padding_bottom)
    });
    PuzzleValueHolder::new(piece)
}

fn indent(puzzle: &PuzzleBuilder, self_indent: f64, inherited_indent: &PuzzlePiece<f64>) -> PuzzleValueHolder<f64> {
    PuzzleValueHolder::new(DerivedPuzzlePiece::new(inherited_indent.clone(),move |v| v + self_indent))
}

pub struct Padder<T> {
    child: Box<T>,
    coord_system: CoordinateSystem,
    /* incoming variables */
    top: PuzzlePiece<f64>,
    inherited_indent: PuzzlePiece<f64>,
    self_indent: f64,
    /* outgoing variables */
    info: PadderInfo,
    height: PuzzleValueHolder<f64>,
    ranges: Arc<Mutex<RangeMerger>>
}

#[derive(Clone)]
pub struct PadderInfo {
    pub child_height: PuzzlePiece<f64>, /* children set this */
    pub draw_top: PuzzleValueHolder<f64>,
    pub indent: PuzzleValueHolder<f64>,
}

impl<T: Clone> Clone for Padder<T> {
    fn clone(&self) -> Self {
        Self {
            child: self.child.clone(),
            ranges: self.ranges.clone(),
            coord_system: self.coord_system.clone(),
            top: self.top.clone(),
            inherited_indent: self.inherited_indent.clone(),
            self_indent: self.self_indent.clone(),
            info: self.info.clone(),
            height: self.height.clone(),
        }
    }
}

fn add_report(metadata: &mut AllotmentMetadata2Builder, key: &str, top: &PuzzleValueHolder<f64>, height: &PuzzleValueHolder<f64>) {
    let mut values = HashMap::new();
    values.insert("type".to_string(), PuzzleValueHolder::new(ConstantPuzzlePiece::new(key.to_string())));
    values.insert("offset".to_string(), PuzzleValueHolder::new(DerivedPuzzlePiece::new(top.clone(),|v| v.to_string())));
    values.insert("height".to_string(), PuzzleValueHolder::new(DerivedPuzzlePiece::new(height.clone(),|v| v.to_string())));
    let group = AllotmentMetadataGroup::new(values);
    metadata.add(group);
}

impl<T> Padder<T> {
    pub fn new<F>(puzzle: &PuzzleBuilder, coord_system: &CoordinateSystem, padding: &Padding, metadata: &mut AllotmentMetadata2Builder, ctor: F) -> Padder<T> where F: FnOnce(&PadderInfo) -> T {
        let mut top = puzzle.new_piece();
        #[cfg(debug_assertions)]
        top.set_name("padder/top");
        let padding_top = padding.padding_top;
        let padding_bottom = padding.padding_bottom;
        let min_height = padding.min_height;
        let mut inherited_indent = puzzle.new_piece_default(0.);
        #[cfg(debug_assertions)]
        inherited_indent.set_name("padder/inherited-indent");
        let self_indent = padding.indent;
        let draw_top = draw_top(&top,padding_top);
        let mut child_height = puzzle.new_piece();
        #[cfg(debug_assertions)]
        child_height.set_name("padder/child-height");
        let height = height(&puzzle,&child_height,min_height,padding_top,padding_bottom);
        let info = PadderInfo {
            draw_top, child_height,
            indent: indent(&puzzle,self_indent,&inherited_indent)
        };
        if let Some(report) = &padding.report {
            add_report(metadata,report,&PuzzleValueHolder::new(top.clone()),&height);
        }
        let child = ctor(&info);
        Padder {
            child: Box::new(child),
            ranges: Arc::new(Mutex::new(RangeMerger::new(puzzle))),
            coord_system: coord_system.clone(),
            top, inherited_indent, self_indent, height,
            info
        }
    }

    pub fn draw_top(&self) -> &PuzzleValueHolder<f64> { &self.info.draw_top }
    pub fn child(&self) -> &T { &self.child }
    pub fn child_mut(&mut self) -> &mut T { &mut self.child }
}

impl<T> Coordinated for Padder<T> {
    fn coordinate_system(&self) -> &CoordinateSystem { &self.coord_system }
}

impl<T> Stackable for Padder<T> {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.top.add_solver(&[value.dependency()], move |solution| {
            Some(value.get_clone(solution))
        });
    }

    fn height(&self) -> PuzzleValueHolder<f64> { self.height.clone() }

    fn set_indent(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.inherited_indent.add_solver(&[value.dependency()], move |solution| {
            Some(value.get_clone(solution))
        });
    }

    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> {
        PuzzleValueHolder::new(self.info.draw_top.clone())
    }

    fn full_range(&self) -> PuzzleValueHolder<RangeUsed<f64>> { lock!(self.ranges).output() }    
}

impl<T: StackableAddable> StackableAddable for Padder<T> {
    fn add_child(&mut self, child: &dyn Stackable, priority: i64) {
        self.child.add_child(child,priority);
        lock!(self.ranges).add(&child.full_range());
    }
}
