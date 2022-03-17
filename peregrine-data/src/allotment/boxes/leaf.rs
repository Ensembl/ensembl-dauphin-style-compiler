use std::{sync::{Arc, Mutex}, borrow::Borrow, collections::{hash_map::RandomState, HashMap}};

use peregrine_toolkit::{lock, puzzle::{PuzzlePiece, PuzzleSolution, PuzzleValueHolder, DerivedPuzzlePiece, PuzzleValue, ClonablePuzzleValue, Puzzle, PuzzleBuilder, ConstantPuzzlePiece}, log};

use crate::{CoordinateSystem, allotment::{core::{rangeused::RangeUsed, arbitrator::{Arbitrator, BpPxConverter}}, transformers::{transformers::{Transformer, TransformerVariety}, simple::{SimpleTransformerHolder, SimpleTransformer}, drawinginfo::DrawingInfo}, style::style::LeafCommonStyle}, CoordinateSystemVariety};

use super::{boxtraits::{Stackable, Transformable, Coordinated }};

fn full_range_piece(puzzle: &PuzzleBuilder, coord_system: &CoordinateSystem, base_range: &PuzzlePiece<RangeUsed<f64>>, pixel_range: &PuzzlePiece<RangeUsed<f64>>, bp_px_converter: &PuzzleValueHolder<Arc<BpPxConverter>>) -> PuzzleValueHolder<RangeUsed<f64>> {
    let base_range = base_range.clone();
    let pixel_range = pixel_range.clone();
    let bp_px_converter = bp_px_converter.clone();
    let coord_system = coord_system.clone();
    let mut piece = puzzle.new_piece_default(RangeUsed::None);
    #[cfg(debug_assertions)]
    piece.set_name("leaf/full_range_piece");
    piece.add_solver(&[base_range.dependency(),pixel_range.dependency(),bp_px_converter.dependency()],move |solution| {
        let base_range = base_range.get_clone(solution);
        let pixel_range = pixel_range.get_clone(solution);
        let bp_px_converter = bp_px_converter.get_clone(solution);
        Some(if coord_system.is_tracking() {
            bp_px_converter.full_pixel_range(&base_range,&pixel_range)
        } else {
            pixel_range
        })
    });
    PuzzleValueHolder::new(piece)
}

#[derive(Clone)]
pub struct FloatingLeaf {
    statics: Arc<LeafCommonStyle>,
    drawing_info: Arc<DrawingInfo>,
    base_range_piece: PuzzlePiece<RangeUsed<f64>>,
    pixel_range_piece: PuzzlePiece<RangeUsed<f64>>,
    full_range_piece: PuzzleValueHolder<RangeUsed<f64>>,
    max_y_piece: PuzzlePiece<f64>,
    top: PuzzlePiece<f64>,
    bottom: PuzzleValueHolder<f64>,
    indent: PuzzlePiece<f64>
}

impl FloatingLeaf {
    pub fn new(puzzle: &PuzzleBuilder, converter: &Arc<BpPxConverter>, statics: &LeafCommonStyle, drawing_info: &DrawingInfo) -> FloatingLeaf {
        let converter = PuzzleValueHolder::new(ConstantPuzzlePiece::new(converter.clone()));
        let mut base_range_piece = puzzle.new_piece();
        #[cfg(debug_assertions)]
        base_range_piece.set_name("FLoatingLeaf/base_range_piece");
        let drawing_info = Arc::new(drawing_info.clone());
        let drawing_info2 = drawing_info.clone();
        base_range_piece.add_ready(|piece| {
            piece.add_solver(&[], move |_solution| {
                Some(drawing_info2.base_range().clone())
            });
        });
        let pixel_range_piece = puzzle.new_piece();
        #[cfg(debug_assertions)]
        base_range_piece.set_name("FLoatingLeaf/pixel_range_piece");
        let drawing_info2 = drawing_info.clone();
        pixel_range_piece.add_ready(|piece| {
            piece.add_solver(&[], move |_solution| {
                Some(drawing_info2.pixel_range().clone())
            });
        });
        let mut max_y_piece = puzzle.new_piece();
        #[cfg(debug_assertions)]
        max_y_piece.set_name("FLoatingLeaf/max_y_piece");
        let drawing_info2 = drawing_info.clone();
        max_y_piece.add_ready(|piece| {
            piece.add_solver(&[], move |_solution| {
                Some(drawing_info2.max_y())
            });
        });
        let mut top = if statics.coord_system.is_dustbin() { puzzle.new_piece_default(0.) } else { puzzle.new_piece() };
        #[cfg(debug_assertions)]
        top.set_name("FLoatingLeaf/top");
        let drawing_info2 = drawing_info.clone();
        let bottom = PuzzleValueHolder::new(DerivedPuzzlePiece::new(top.clone(),move |top| {
            top + drawing_info2.max_y()
        }));
        let full_range_piece = full_range_piece(
            puzzle,
            &statics.coord_system,&base_range_piece,&pixel_range_piece,&converter);
        let mut indent = puzzle.new_piece_default(0.);
        #[cfg(debug_assertions)]
        indent.set_name("FLoatingLeaf/indent");
        FloatingLeaf {
            statics: Arc::new(statics.clone()),
            drawing_info,
            base_range_piece, pixel_range_piece, max_y_piece, full_range_piece, indent,
            top, bottom
        }
    }

    pub fn leaf_common(&self) -> &LeafCommonStyle { &self.statics }
}

impl Stackable for FloatingLeaf {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.top.add_solver(&[value.dependency()], move |solution| {
            Some(value.get_clone(solution))
        })
    }

    fn set_indent(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.indent.add_solver(&[value.dependency()], move |solution| {
            Some(value.get_clone(solution))
        })
    }

    fn height(&self) -> PuzzleValueHolder<f64> { PuzzleValueHolder::new(self.max_y_piece.clone()) }

    fn top_anchor(&self, _puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> {
        PuzzleValueHolder::new(self.top.clone())
    }

    fn full_range(&self) -> PuzzleValueHolder<RangeUsed<f64>> { 
        if self.statics.coord_system.is_tracking() {
            self.full_range_piece.clone()
        } else {
            PuzzleValueHolder::new(ConstantPuzzlePiece::new(RangeUsed::None))
        }
    }
}

impl Transformable for FloatingLeaf {
    fn cloned(&self) -> Arc<dyn Transformable> {
        Arc::new(self.clone())
    }

    fn make(&self, solution: &PuzzleSolution) -> Arc<dyn Transformer> {
        Arc::new(AnchoredLeaf::new(solution,self))
    }

    fn get_style(&self) -> &LeafCommonStyle { &self.statics }
}

impl Coordinated for FloatingLeaf {
    fn coordinate_system(&self) -> &CoordinateSystem { &self.statics.coord_system }
}

#[cfg_attr(test,derive(Debug))]
#[derive(Clone)]
pub struct AnchoredLeaf {
    statics: Arc<LeafCommonStyle>,
    top: f64,
    height: f64,
    indent: f64
}

impl AnchoredLeaf {
    pub fn new(solution: &PuzzleSolution, floating: &FloatingLeaf) -> AnchoredLeaf {
        AnchoredLeaf {
            statics: floating.statics.clone(),
            top: floating.top.get_clone(solution),
            height: floating.max_y_piece.get_clone(solution),
            indent: floating.indent.get_clone(solution)
        }
    }
}

impl Transformer for AnchoredLeaf {
    fn choose_variety(&self) -> (TransformerVariety,CoordinateSystem) { (TransformerVariety::SimpleTransformer,self.statics.coord_system.clone()) }
    fn into_simple_transformer(&self) -> SimpleTransformerHolder { SimpleTransformerHolder(Arc::new(self.clone())) }
    fn get_style(&self) -> &LeafCommonStyle { &self.statics }

    #[cfg(test)]
    fn describe(&self) -> String {
        format!("{:?}",self)
    }
}

impl SimpleTransformer for AnchoredLeaf {
    fn top(&self) -> f64 { self.top }
    fn bottom(&self) -> f64 { self.top + self.height }
    fn indent(&self) -> f64 { self.indent }
    fn as_simple_transformer(&self) -> &dyn SimpleTransformer { self }
    fn get_style(&self) -> &LeafCommonStyle { &self.statics }
}
