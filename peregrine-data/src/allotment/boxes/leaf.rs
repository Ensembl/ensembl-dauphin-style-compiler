use std::{sync::{Arc}};

use peregrine_toolkit::{puzzle::{DelayedSetter, constant, derived, StaticValue, StaticAnswer, promise_delayed, cache_constant_clonable}};

use crate::{CoordinateSystem, allotment::{core::{aligner::Aligner, carriageoutput::BoxPositionContext}, transformers::{transformers::{Transformer, TransformerVariety}, simple::{SimpleTransformerHolder, SimpleTransformer}, drawinginfo::DrawingInfo}, style::{style::LeafCommonStyle, allotmentname::{AllotmentNamePart, AllotmentName}}, util::{rangeused::RangeUsed, bppxconverter::BpPxConverter}}};

use super::{boxtraits::{Stackable, Transformable, Coordinated, BuildSize }};

// TODO ranged bppxconverter
fn full_range_piece(coord_system: &CoordinateSystem, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>, bp_px_converter: &StaticValue<Arc<BpPxConverter>>) -> StaticValue<RangeUsed<f64>> {
    let base_range = base_range.clone();
    let pixel_range = pixel_range.clone();
    let bp_px_converter = bp_px_converter.clone();
    let coord_system = coord_system.clone();
    cache_constant_clonable(derived(bp_px_converter,move |bp_px_converter| {
        if coord_system.is_tracking() {
            bp_px_converter.full_pixel_range(&base_range,&pixel_range)
        } else {
            pixel_range.clone()
        }
    }))
}

#[derive(Clone)]
pub struct FloatingLeaf {
    name: AllotmentName,
    statics: Arc<LeafCommonStyle>,
//    pixel_range_piece: StaticValue<RangeUsed<f64>>,
  //  pixel_range_piece_setter: DelayedSetter<'static,'static,RangeUsed<f64>>,
    //base_range_piece: StaticValue<RangeUsed<f64>>,
    //base_range_piece_setter: DelayedSetter<'static,'static,RangeUsed<f64>>,
    max_y_piece: StaticValue<f64>,
    max_y_piece_setter: DelayedSetter<'static,'static,f64>,
    converter: StaticValue<Arc<BpPxConverter>>,
    top_setter: Option<DelayedSetter<'static,'static,f64>>,
    top: StaticValue<f64>,
    indent: StaticValue<f64>,
    drawing_info: Arc<DrawingInfo>
}

impl FloatingLeaf {
    pub fn new(name: &AllotmentNamePart, converter: &Arc<BpPxConverter>, statics: &LeafCommonStyle, drawing_info: &DrawingInfo, aligner: &Aligner) -> FloatingLeaf {
        let drawing_info = Arc::new(drawing_info.clone());
        let (max_y_piece_setter,max_y_piece) = promise_delayed();
        if statics.coord_system.is_dustbin() {
            max_y_piece_setter.set(constant(0.));
        }
        let (top_setter,top) = if statics.coord_system.is_dustbin() {
            (None,constant(0.))
        } else {
            let (setter,value) = promise_delayed();
            (Some(setter),value)
        };
        let indent = aligner.get(&statics.indent);
        FloatingLeaf {
            name: AllotmentName::from_part(name),
            statics: Arc::new(statics.clone()),
            converter: constant(converter.clone()), // kept in puzzle because SHOULD be variable
            max_y_piece, max_y_piece_setter,
            indent, 
            top_setter, top,
            drawing_info
        }
    }

    fn full_range(&self, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>, bp_px_converter: &StaticValue<Arc<BpPxConverter>>) -> StaticValue<RangeUsed<f64>> { 
        let full_range_piece = full_range_piece(
            &self.statics.coord_system,&base_range,&pixel_range,bp_px_converter);
        if self.statics.coord_system.is_tracking() && !self.statics.bump_invisible {
            full_range_piece.clone()
        } else {
            constant(RangeUsed::None)
        }
    }
}

impl Stackable for FloatingLeaf {
    fn priority(&self) -> i64 { self.statics.priority }
    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
    fn name(&self) -> &AllotmentName { &self.name }

    fn build(&mut self, _prep: &mut BoxPositionContext) -> BuildSize {
        self.max_y_piece_setter.set(constant(self.drawing_info.max_y()));
        BuildSize {
            name: self.name.clone(),
            height: self.max_y_piece.clone(),
            range: self.full_range(self.drawing_info.base_range(),self.drawing_info.pixel_range(),&self.converter)
        }
    }

    fn locate(&mut self, _prep: &mut BoxPositionContext, value: &StaticValue<f64>) {
        let value = value.clone();
        if let Some(top_setter) = &self.top_setter {
            top_setter.set(value.clone());
        }
    }
}

impl Transformable for FloatingLeaf {
    fn cloned(&self) -> Arc<dyn Transformable> {
        Arc::new(self.clone())
    }

    fn make(&self, answer_index: &StaticAnswer) -> Arc<dyn Transformer> {
        Arc::new(AnchoredLeaf::new(answer_index,self))
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
    pub fn new(answer_index: &StaticAnswer, floating: &FloatingLeaf) -> AnchoredLeaf {
        AnchoredLeaf {
            statics: floating.statics.clone(),
            top: floating.top.call(answer_index),
            height: floating.max_y_piece.call(answer_index),
            indent: floating.indent.call(answer_index)
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
