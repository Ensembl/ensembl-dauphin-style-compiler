use std::{collections::HashMap, sync::{Arc, Mutex}, borrow::BorrowMut};

use peregrine_toolkit::{lock, puzzle::{Puzzle, PuzzleSolution, PuzzlePiece, ClonablePuzzleValue, PuzzleValue, PuzzleValueHolder, PuzzleBuilder, ConstantPuzzlePiece}};

use crate::{CarriageExtent, allotment::tree::collisionalgorithm::CollisionAlgorithmHolder, ShapeRequest, ShapeRequestGroup};

use super::rangeused::RangeUsed;

/* The Arbitrator stores the offsets of other elements for alingment.
 */


pub struct BpPxConverter {
    max_px_per_bp: Option<f64>,
    bp_start: f64
}

impl BpPxConverter {
    fn real_calc_max_px_per_bp(request: &ShapeRequestGroup) -> f64 {
        let bp_per_carriage = request.region().scale().bp_in_carriage() as f64;
        let max_px_per_carriage = request.pixel_size().max_px_per_carriage() as f64;
        max_px_per_carriage / bp_per_carriage
    }

    fn calc_max_px_per_bp(extent: Option<&ShapeRequestGroup>) -> Option<f64> {
        extent.map(|e| BpPxConverter::real_calc_max_px_per_bp(e))
    }

    pub(crate) fn new(extent: Option<&ShapeRequestGroup>) -> BpPxConverter {
        BpPxConverter {
            max_px_per_bp: BpPxConverter::calc_max_px_per_bp(extent),
            bp_start: extent.map(|x| x.region().min_value() as f64).unwrap_or(0.)
        }
    }

    fn real_calc_max_px_per_bp2(request: &ShapeRequest) -> f64 {
        let bp_per_carriage = request.region().scale().bp_in_carriage() as f64;
        let max_px_per_carriage = request.pixel_size().max_px_per_carriage() as f64;
        max_px_per_carriage / bp_per_carriage
    }

    fn calc_max_px_per_bp2(extent: Option<&ShapeRequest>) -> Option<f64> {
        extent.map(|e| BpPxConverter::real_calc_max_px_per_bp2(e))
    }

    pub(crate) fn new2(extent: Option<&ShapeRequest>) -> BpPxConverter {
        BpPxConverter {
            max_px_per_bp: BpPxConverter::calc_max_px_per_bp2(extent),
            bp_start: extent.map(|x| x.region().min_value() as f64).unwrap_or(0.)
        }
    }

    pub fn full_pixel_range(&self, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>) -> RangeUsed<f64> {
        if let Some(max_px_per_bp) = self.max_px_per_bp {
            base_range.plus_scalar(-self.bp_start).pixel_range(pixel_range,max_px_per_bp)
        } else {
            pixel_range.clone()
        }
    }
}
