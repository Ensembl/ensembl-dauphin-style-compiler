use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzleBuilder, DerivedPuzzlePiece, ConstantPuzzlePiece, DelayedPuzzleValue, compose2, CommutingSequence}, lock};

use crate::{allotment::{core::{allotmentmetadata::{AllotmentMetadataBuilder, AllotmentMetadataGroup}, aligner::Aligner, carriageuniverse::CarriageUniversePrep}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentName, AllotmentNamePart}}, boxes::boxtraits::Stackable, util::rangeused::RangeUsed}, CoordinateSystem};

use super::{boxtraits::{Coordinated, BuildSize}};

fn draw_top(top: &DelayedPuzzleValue<f64>, padding_top: f64) -> PuzzleValueHolder<f64> {
    PuzzleValueHolder::new(DerivedPuzzlePiece::new(top.clone(),move |top| *top + padding_top))
}

fn height(puzzle: &PuzzleBuilder, child_height: &PuzzleValueHolder<f64>, tracked_height: &DelayedPuzzleValue<f64>, min_height: f64, padding_top: f64, padding_bottom: f64) -> PuzzleValueHolder<f64> {
    compose2(puzzle,child_height,&PuzzleValueHolder::new(tracked_height.clone()),move |child_height,tracked_height| {
        let internal_height = child_height.max(min_height);
        let external_height = padding_top + internal_height + padding_bottom;
        external_height.max(*tracked_height)
    })
}

pub(crate) trait PadderSpecifics {
    fn cloned(&self) -> Box<dyn PadderSpecifics>;
    fn add_child(&mut self, child: &dyn Stackable);
    fn build_reduce(&mut self, children: &[(&Box<dyn Stackable>,BuildSize)]) -> PuzzleValueHolder<f64>;
}

pub struct Padder {
    builder: PuzzleBuilder,
    specifics: Box<dyn PadderSpecifics>,
    children: Arc<Mutex<Vec<Box<dyn Stackable>>>>,
    coord_system: CoordinateSystem,
    priority: i64,
    /* incoming variables */
    top: DelayedPuzzleValue<f64>,
    /* outgoing variables */
    name: AllotmentName,
    info: PadderInfo,
    height: DelayedPuzzleValue<f64>,
    style: Arc<ContainerAllotmentStyle>,
}

#[derive(Clone)]
pub struct PadderInfo {
    pub draw_top: PuzzleValueHolder<f64>
}

impl Clone for Padder {
    fn clone(&self) -> Self {
        Self {
            builder: self.builder.clone(),
            specifics: self.specifics.cloned(),
            children: self.children.clone(),
            coord_system: self.coord_system.clone(),
            priority: self.priority.clone(),
            top: self.top.clone(),
            info: self.info.clone(),
            height: self.height.clone(),
            name: self.name.clone(),
            style: self.style.clone()
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

impl Padder {
    pub(crate) fn new<F>(prep: &mut CarriageUniversePrep, name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner, ctor: F) -> Padder where F: FnOnce(&mut CarriageUniversePrep, &PadderInfo) -> Box<dyn PadderSpecifics> {
        let top = DelayedPuzzleValue::new(&prep.puzzle);
        let draw_top = draw_top(&top,style.padding.padding_top);
        let info = PadderInfo {
            draw_top
        };
        let specifics = ctor(prep,&info);
        if let Some(datum) = &style.set_align {
            aligner.set_datum(&prep.puzzle,datum,&PuzzleValueHolder::new(top.clone()));
        }
        let height = DelayedPuzzleValue::new(&prep.puzzle);
        Padder {
            builder: prep.puzzle.clone(),
            name: AllotmentName::from_part(name),
            specifics,
            children: Arc::new(Mutex::new(vec![])),
            coord_system: style.coord_system.clone(),
            priority: style.priority,
            top,
            height,
            info,
            style: Arc::new(style.clone())
        }
    }

    pub(crate) fn add_child(&mut self, child: &dyn Stackable) {
        self.specifics.add_child(child);
        lock!(self.children).push(child.cloned());
    }
}

impl Coordinated for Padder {
    fn coordinate_system(&self) -> &CoordinateSystem { &self.coord_system }
}

impl Stackable for Padder {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.top.set(&self.builder,value);
    }

    fn build(&mut self, prep: &mut CarriageUniversePrep) -> BuildSize {
        let mut ranges = CommutingSequence::new(RangeUsed::None, |x,y| x.merge(&y));
        let mut children = lock!(self.children);
        let mut input = vec![];
        for child in &mut *children {
            let size = child.build(prep);
            ranges.add(&size.range);
            input.push((&*child,size));
        }
        let tracked_height = DelayedPuzzleValue::new(&prep.puzzle);
        let internal = self.specifics.build_reduce(&input);
        let height = height(&self.builder,&internal,&tracked_height,self.style.padding.min_height,self.style.padding.padding_top,self.style.padding.padding_bottom);
        if self.style.tracked_height {
            prep.height_tracker.add(&self.name,&height);
            let global_piece = prep.height_tracker.get_piece(&self.name).clone();
            tracked_height.set(&mut prep.puzzle,PuzzleValueHolder::new(global_piece));
        } else {
            tracked_height.set(&mut prep.puzzle,PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)));
        }
        if let Some(report) = &self.style.padding.report {
            add_report(&mut prep.metadata,report,&PuzzleValueHolder::new(self.top.clone()),&height);
        }
        self.height.set(&prep.puzzle,height.clone());
        BuildSize {
            height,
            range: ranges.build(&prep.puzzle)
        }
    }

    fn priority(&self) -> i64 { self.priority }

    fn top_anchor(&self, _puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> {
        PuzzleValueHolder::new(self.info.draw_top.clone())
    }

    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
}
