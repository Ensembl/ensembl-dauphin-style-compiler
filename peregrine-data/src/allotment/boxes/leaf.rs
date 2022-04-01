use std::{sync::{Arc}};

use peregrine_toolkit::{puzzle::{PuzzleSolution, PuzzleValueHolder, ClonablePuzzleValue, PuzzleBuilder, ConstantPuzzlePiece, DelayedPuzzleValue, DelayedConstant, DerivedPuzzlePiece}};

use crate::{CoordinateSystem, allotment::{core::{aligner::Aligner, carriageuniverse::CarriageUniversePrep}, transformers::{transformers::{Transformer, TransformerVariety}, simple::{SimpleTransformerHolder, SimpleTransformer}, drawinginfo::DrawingInfo}, style::{style::LeafCommonStyle, allotmentname::{AllotmentNamePart, AllotmentName}}, util::{rangeused::RangeUsed, bppxconverter::BpPxConverter}}};

use super::{boxtraits::{Stackable, Transformable, Coordinated, BuildSize }};

// TODO ranged bppxconverter
fn full_range_piece(coord_system: &CoordinateSystem, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>, bp_px_converter: &PuzzleValueHolder<Arc<BpPxConverter>>) -> PuzzleValueHolder<RangeUsed<f64>> {
    let base_range = base_range.clone();
    let pixel_range = pixel_range.clone();
    let bp_px_converter = bp_px_converter.clone();
    let coord_system = coord_system.clone();
    PuzzleValueHolder::new(DerivedPuzzlePiece::new(bp_px_converter,move |bp_px_converter| {
        if coord_system.is_tracking() {
            bp_px_converter.full_pixel_range(&base_range,&pixel_range)
        } else {
            pixel_range.clone()
        }
    }))
}

#[derive(Clone)]
pub struct FloatingLeaf {
    builder: PuzzleBuilder,
    name: AllotmentName,
    statics: Arc<LeafCommonStyle>,
    pixel_range_piece: DelayedConstant<RangeUsed<f64>>,
    base_range_piece: DelayedConstant<RangeUsed<f64>>,
    converter: PuzzleValueHolder<Arc<BpPxConverter>>,
    max_y_piece: DelayedConstant<f64>,
    top: Option<DelayedPuzzleValue<f64>>,
    top_value: PuzzleValueHolder<f64>,
    indent: PuzzleValueHolder<f64>,
    drawing_info: Arc<DrawingInfo>
}

impl FloatingLeaf {
    pub fn new(puzzle: &PuzzleBuilder, name: &AllotmentNamePart, converter: &Arc<BpPxConverter>, statics: &LeafCommonStyle, drawing_info: &DrawingInfo, aligner: &Aligner) -> FloatingLeaf {
        let drawing_info = Arc::new(drawing_info.clone());
        let base_range_piece = DelayedConstant::new();
        let pixel_range_piece = DelayedConstant::new();
        let max_y_piece = DelayedConstant::new();
        if statics.coord_system.is_dustbin() {
            base_range_piece.set(RangeUsed::None);
            pixel_range_piece.set(RangeUsed::None);
            max_y_piece.set(0.);
        }
        let (top,top_value) = if statics.coord_system.is_dustbin() {
            (None,PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)))
        } else {
            let top = DelayedPuzzleValue::new(puzzle);
            let top_value = PuzzleValueHolder::new(top.clone());
            (Some(top),top_value)
        };
        let indent = aligner.get(puzzle,&statics.indent);
        FloatingLeaf {
            name: AllotmentName::from_part(name),
            builder: puzzle.clone(),
            statics: Arc::new(statics.clone()),
            converter: PuzzleValueHolder::new(ConstantPuzzlePiece::new(converter.clone())), // kept in puzzle because SHOULD be variable
            max_y_piece,
            indent, pixel_range_piece, base_range_piece,
            top, top_value,
            drawing_info
        }
    }

    fn full_range(&self, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>, bp_px_converter: &PuzzleValueHolder<Arc<BpPxConverter>>) -> PuzzleValueHolder<RangeUsed<f64>> { 
        let full_range_piece = full_range_piece(
            &self.statics.coord_system,&base_range,&pixel_range,bp_px_converter);
        if self.statics.coord_system.is_tracking() && !self.statics.bump_invisible {
            full_range_piece.clone()
        } else {
            PuzzleValueHolder::new(ConstantPuzzlePiece::new(RangeUsed::None))
        }
    }
}

impl Stackable for FloatingLeaf {
    fn priority(&self) -> i64 { self.statics.priority }
    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
    fn name(&self) -> &AllotmentName { &self.name }

    fn build(&mut self, _prep: &mut CarriageUniversePrep) -> BuildSize {
        self.pixel_range_piece.set(self.drawing_info.pixel_range().clone());
        self.base_range_piece.set(self.drawing_info.base_range().clone());
        self.max_y_piece.set(self.drawing_info.max_y());
        BuildSize {
            name: self.name.clone(),
            height: PuzzleValueHolder::new(self.max_y_piece.clone()),
            range: self.full_range(self.drawing_info.base_range(),self.drawing_info.pixel_range(),&self.converter)
        }
    }

    fn locate(&mut self, _prep: &mut CarriageUniversePrep, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        if let Some(top) = &self.top {
            top.set(&self.builder,value.clone());
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
