use std::{sync::{Arc}};
use peregrine_toolkit::{puzzle::{DelayedSetter, constant, StaticValue, StaticAnswer, promise_delayed, delayed }};
use crate::{allotment::{core::{allotmentname::{AllotmentName, AllotmentNamePart}, boxtraits::{ContainerOrLeaf, BuildSize }, boxpositioncontext::BoxPositionContext, drawinginfo::DrawingInfo}, style::{style::{LeafStyle, Indent}}, util::{rangeused::RangeUsed, bppxconverter::BpPxConverter}, globals::playingfield::PlayingFieldEdge, stylespec::stylegroup::AllStylesForProgram}, CoordinateSystem, LeafRequest};

// TODO ranged bppxconverter
fn full_range_piece(coord_system: &CoordinateSystem, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>, bp_px_converter: &Arc<BpPxConverter>) -> RangeUsed<f64> {
    let base_range = base_range.clone();
    let pixel_range = pixel_range.clone();
    let bp_px_converter = bp_px_converter.clone();
    let coord_system = coord_system.clone();
    if coord_system.is_tracking() {
        bp_px_converter.full_carriage_range(&base_range,&pixel_range)
    } else {
        pixel_range.clone()
    }
}

#[derive(Clone)]
pub struct FloatingLeaf {
    name: AllotmentName,
    statics: Arc<LeafStyle>,
    max_y_piece: StaticValue<f64>,
    indent: StaticValue<Option<f64>>,
    indent_setter: DelayedSetter<'static,'static,f64>,
    max_y_piece_setter: DelayedSetter<'static,'static,f64>,
    top_setter: Option<DelayedSetter<'static,'static,f64>>,
    top: StaticValue<f64>,
    drawing_info: Arc<DrawingInfo>
}

impl FloatingLeaf {
    pub fn new(name: &AllotmentNamePart, statics: &LeafStyle, drawing_info: &DrawingInfo) -> FloatingLeaf {
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
        let (indent_setter,indent) = delayed();
        FloatingLeaf {
            name: AllotmentName::from_part(name),
            statics: Arc::new(statics.clone()),
            max_y_piece, max_y_piece_setter,
            top_setter, top,
            indent,
            indent_setter,
            drawing_info
        }
    }

    fn full_range(&self, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>, bp_px_converter: &Arc<BpPxConverter>) -> RangeUsed<f64> { 
        let full_range_piece = full_range_piece(
            &self.statics.coord_system,&base_range,&pixel_range,bp_px_converter);
        if self.statics.coord_system.is_tracking() && !self.statics.bump_invisible {
            full_range_piece.clone()
        } else {
            RangeUsed::None
        }
    }
}

impl ContainerOrLeaf for FloatingLeaf {
    fn anchor_leaf(&self, answer_index: &StaticAnswer) -> Option<AnchoredLeaf> {
        Some(AnchoredLeaf::new(answer_index,self))
    }

    fn get_leaf(&mut self, _pending: &LeafRequest, _cursor: usize, _styles: &Arc<AllStylesForProgram>) -> FloatingLeaf {
        panic!("get_leaf called on leaf!");
    }
    
    fn coordinate_system(&self) -> &CoordinateSystem { &self.statics.coord_system }
    fn priority(&self) -> i64 { self.statics.priority }
    fn name(&self) -> &AllotmentName { &self.name }

    fn build(&mut self, prep: &mut BoxPositionContext) -> BuildSize {
        self.max_y_piece_setter.set(constant(self.drawing_info.max_y()));
        BuildSize {
            name: self.name.clone(),
            height: self.max_y_piece.clone(),
            range: self.full_range(self.drawing_info.base_range(),self.drawing_info.pixel_range(),&prep.bp_px_converter)
        }
    }

    fn locate(&mut self, prep: &mut BoxPositionContext, value: &StaticValue<f64>) {
        let sr = &mut prep.state_request;
        let indent = match &self.statics.indent {
            Indent::None => None,
            Indent::Top => Some(sr.playing_field_mut().global(&PlayingFieldEdge::Top)),
            Indent::Left => Some(sr.playing_field_mut().global(&PlayingFieldEdge::Left)),
            Indent::Bottom => Some(sr.playing_field_mut().global(&PlayingFieldEdge::Bottom)),
            Indent::Right => Some(sr.playing_field_mut().global(&PlayingFieldEdge::Right)),
            Indent::Datum(name) => Some(sr.aligner_mut().global(name))
        };
        if let Some(indent) = indent {
            self.indent_setter.set(indent.clone());
        }
        let value = value.clone();
        if let Some(top_setter) = &self.top_setter {
            top_setter.set(value.clone());
        }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct AnchoredLeaf {
    statics: Arc<LeafStyle>,
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
            indent: floating.indent.call(answer_index).unwrap_or(0.)
        }
    }

    pub(crate) fn coordinate_system(&self) -> &CoordinateSystem { &self.statics.coord_system }
    pub(crate) fn get_style(&self) -> &LeafStyle { &self.statics }

    #[cfg(any(debug_assertions,test))]
    pub(crate) fn describe(&self) -> String {
        format!("{:?}",self)
    }

    pub(crate) fn top(&self) -> f64 { self.top }
    pub(crate) fn bottom(&self) -> f64 { self.top + self.height }
    pub(crate) fn indent(&self) -> f64 { self.indent }
}
