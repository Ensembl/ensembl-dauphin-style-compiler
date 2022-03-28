use std::{sync::{Arc}};

use peregrine_toolkit::{puzzle::{PuzzleSolution, PuzzleValueHolder, PuzzleValue, ClonablePuzzleValue, PuzzleBuilder, ConstantPuzzlePiece, DelayedPuzzleValue, DelayedConstant}};

use crate::{CoordinateSystem, allotment::{core::{aligner::Aligner}, transformers::{transformers::{Transformer, TransformerVariety}, simple::{SimpleTransformerHolder, SimpleTransformer}, drawinginfo::DrawingInfo}, style::style::LeafCommonStyle, util::{rangeused::RangeUsed, bppxconverter::BpPxConverter}}};

use super::{boxtraits::{Stackable, Transformable, Coordinated }};

fn full_range_piece(puzzle: &PuzzleBuilder, coord_system: &CoordinateSystem, base_range: &DelayedConstant<RangeUsed<f64>>, pixel_range: &DelayedConstant<RangeUsed<f64>>, bp_px_converter: &PuzzleValueHolder<Arc<BpPxConverter>>) -> PuzzleValueHolder<RangeUsed<f64>> {
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
    builder: PuzzleBuilder,
    statics: Arc<LeafCommonStyle>,
    full_range_piece: PuzzleValueHolder<RangeUsed<f64>>,
    max_y_piece: PuzzleValueHolder<f64>,
    top: Option<DelayedPuzzleValue<f64>>,
    top_value: PuzzleValueHolder<f64>,
    indent: PuzzleValueHolder<f64>
}

impl FloatingLeaf {
    pub fn new(puzzle: &PuzzleBuilder, converter: &Arc<BpPxConverter>, statics: &LeafCommonStyle, drawing_info: &DrawingInfo, aligner: &Aligner) -> FloatingLeaf {
        let converter = PuzzleValueHolder::new(ConstantPuzzlePiece::new(converter.clone()));
        let drawing_info = Arc::new(drawing_info.clone());
        let drawing_info2 = drawing_info.clone();
        let base_range_piece = DelayedConstant::new();
        let piece = base_range_piece.clone();
        puzzle.add_ready(move |_| {
            piece.set(drawing_info2.base_range().clone());
        });
        let pixel_range_piece = DelayedConstant::new();
        let piece = pixel_range_piece.clone();
        let drawing_info2 = drawing_info.clone();
        puzzle.add_ready(move |_| {
            piece.set(drawing_info2.pixel_range().clone());
        });
        let max_y_piece = DelayedConstant::new();
        let piece = max_y_piece.clone();
        let drawing_info2 = drawing_info.clone();
        puzzle.add_ready(move |_| {
            piece.set(drawing_info2.max_y());
        });
        let (top,top_value) = if statics.coord_system.is_dustbin() {
            (None,PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)))
        } else {
            let top = DelayedPuzzleValue::new(puzzle);
            let top_value = PuzzleValueHolder::new(top.clone());
            (Some(top),top_value)
        };
        let full_range_piece = full_range_piece(
            puzzle,
            &statics.coord_system,&base_range_piece,&pixel_range_piece,&converter);
        let indent = aligner.get(puzzle,&statics.indent);
        FloatingLeaf {
            builder: puzzle.clone(),
            statics: Arc::new(statics.clone()),
            max_y_piece: PuzzleValueHolder::new(max_y_piece),
            full_range_piece, indent,
            top, top_value
        }
    }
}

impl Stackable for FloatingLeaf {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        if let Some(top) = &self.top {
            top.set(&self.builder,value.clone());
        }
    }

    fn height(&self) -> PuzzleValueHolder<f64> { self.max_y_piece.clone() }

    fn top_anchor(&self, _puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.top_value.clone() }

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

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
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
            top: floating.top_value.get_clone(solution),
            height: floating.max_y_piece.get_clone(solution),
            indent: floating.indent.get_clone(solution)
        }
    }
}

impl Transformer for AnchoredLeaf {
    fn choose_variety(&self) -> (TransformerVariety,CoordinateSystem) { (TransformerVariety::SimpleTransformer,self.statics.coord_system.clone()) }
    fn into_simple_transformer(&self) -> Option<SimpleTransformerHolder> { Some(SimpleTransformerHolder(Arc::new(self.clone()))) }
    fn get_style(&self) -> &LeafCommonStyle { &self.statics }

    #[cfg(any(debug_assertions,test))]
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
