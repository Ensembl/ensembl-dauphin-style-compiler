use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzleBuilder, PuzzlePiece, DerivedPuzzlePiece, ClonablePuzzleValue, PuzzleValue, ConstantPuzzlePiece, FoldValue}, log, lock};

use crate::{allotment::{core::{allotmentmetadata2::{AllotmentMetadata2Builder, AllotmentMetadataGroup}, rangeused::RangeUsed, aligner::Aligner}, style::{style::{Padding, ContainerAllotmentStyle}}, boxes::boxtraits::Stackable}, CoordinateSystem};

use super::{boxtraits::{Coordinated, StackableAddable}};

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
    ranges: Arc<Mutex<FoldValue<RangeUsed<f64>>>>,
    full_range: PuzzleValueHolder<RangeUsed<f64>>
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
            full_range: self.full_range.clone()
        }
    }
}

fn add_report(metadata: &mut AllotmentMetadata2Builder, in_values: &HashMap<String,String>, top: &PuzzleValueHolder<f64>, height: &PuzzleValueHolder<f64>) {
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
    pub fn new<F>(puzzle: &PuzzleBuilder, coord_system: &CoordinateSystem, style: &ContainerAllotmentStyle, metadata: &mut AllotmentMetadata2Builder, aligner: &Aligner, ctor: F) -> Padder<T> where F: FnOnce(&PadderInfo) -> T {
        let mut top = puzzle.new_piece();
        #[cfg(debug_assertions)]
        top.set_name("padder/top");
        let padding_top = style.padding.padding_top;
        let padding_bottom = style.padding.padding_bottom;
        let min_height = style.padding.min_height;
        let mut inherited_indent = puzzle.new_piece_default(0.);
        #[cfg(debug_assertions)]
        inherited_indent.set_name("padder/inherited-indent");
        let self_indent = style.padding.indent;
        let draw_top = draw_top(&top,padding_top);
        let mut child_height = puzzle.new_piece();
        #[cfg(debug_assertions)]
        child_height.set_name("padder/child-height");
        let height = height(&puzzle,&child_height,min_height,padding_top,padding_bottom);
        let info = PadderInfo {
            draw_top, child_height,
            indent: indent(&puzzle,self_indent,&inherited_indent)
        };
        if let Some(report) = &style.padding.report {
            add_report(metadata,report,&PuzzleValueHolder::new(top.clone()),&height);
        }
        let child = ctor(&info);
        let mut full_range = puzzle.new_piece();
        let ranges = Arc::new(Mutex::new(FoldValue::new(full_range.clone(), |x : RangeUsed<f64>,y| x.merge(&y))));
        let ranges2 = ranges.clone();
        puzzle.add_ready(move |_| lock!(ranges2).build());
        #[cfg(debug_assertions)]
        full_range.set_name("padder/full_range");
        if let Some(datum) = &style.set_align {
            aligner.set_datum(puzzle,datum,&PuzzleValueHolder::new(top.clone()));
        }
        Padder {
            child: Box::new(child),
            ranges,
            coord_system: coord_system.clone(),
            top, inherited_indent, self_indent, height,
            info, 
            full_range: PuzzleValueHolder::new(full_range.clone())
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

    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> {
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
