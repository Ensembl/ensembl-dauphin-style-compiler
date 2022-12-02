use std::sync::Arc;
use peregrine_toolkit::puzzle::{StaticValue, DelayedSetter, promise_delayed, constant, delayed, StaticAnswer};
use crate::{allotment::{core::{allotmentname::AllotmentName, leafshapebounds::LeafShapeBounds, boxpositioncontext::BoxPositionContext}, style::{leafstyle::{LeafStyle, Indent}, styletree::StyleTree}, util::{rangeused::RangeUsed, bppxconverter::BpPxConverter}, layout::stylebuilder::{ContainerOrLeaf, BuildSize}}, LeafRequest, CoordinateSystem, globals::playingfield::PlayingFieldEdge};
use super::anchored::AnchoredLeaf;

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
    pub(super) statics: Arc<LeafStyle>,
    pub(super) max_y_piece: StaticValue<f64>,
    pub(super) indent: StaticValue<Option<f64>>,
    indent_setter: DelayedSetter<'static,'static,f64>,
    max_y_piece_setter: DelayedSetter<'static,'static,f64>,
    top_setter: Option<DelayedSetter<'static,'static,f64>>,
    pub(super) top: StaticValue<f64>,
    drawing_info: Arc<LeafShapeBounds>
}

impl FloatingLeaf {
    pub fn new(name: &AllotmentName, statics: &LeafStyle, drawing_info: &LeafShapeBounds) -> FloatingLeaf {
        let drawing_info = Arc::new(drawing_info.clone());
        let (max_y_piece_setter,max_y_piece) = promise_delayed();
        if statics.aux.coord_system.is_dustbin() {
            max_y_piece_setter.set(constant(0.));
        }
        let (top_setter,top) = if statics.aux.coord_system.is_dustbin() {
            (None,constant(0.))
        } else {
            let (setter,value) = promise_delayed();
            (Some(setter),value)
        };
        let (indent_setter,indent) = delayed();
        FloatingLeaf {
            name: name.clone(),
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
            &self.statics.aux.coord_system,&base_range,&pixel_range,bp_px_converter);
        if self.statics.aux.coord_system.is_tracking() && !self.statics.bump_invisible {
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

    fn get_leaf(&mut self, _pending: &LeafRequest, _cursor: usize, _styles: &Arc<StyleTree>) -> FloatingLeaf {
        panic!("get_leaf called on leaf!");
    }
    
    fn coordinate_system(&self) -> &CoordinateSystem { &self.statics.aux.coord_system }
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
