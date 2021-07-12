use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::layer::Layer;
use crate::shape::layers::patina::PatinaYielder;
use crate::webgl::{AttribHandle, ProcessStanzaAddable, ProcessStanzaElements, ProgramBuilder};
use peregrine_data::{
    Allotment, AllotmentPosition, Flattenable, HoleySpaceBase, HoleySpaceBaseArea, HollowEdge, PositionVariant,
    SpaceBase, SpaceBaseArea, SpaceBaseAreaParameterLocation, SpaceBaseParameterLocation, Substitutions, VariableValues
};
use super::super::util::arrayutil::rectangle64;
use super::triangleskind::TrianglesKind;
use super::trianglesprogramlink::TrianglesProgramLink;
use super::trianglesyielder::TrackTrianglesYielder;
use crate::util::message::Message;

enum RectanglesLocation {
    Area(SpaceBaseArea<f64>,Substitutions<SpaceBaseAreaParameterLocation>,Option<HollowEdge<f64>>),
    Sized(SpaceBase<f64>,Substitutions<SpaceBaseParameterLocation>,Vec<f64>,Vec<f64>)
}

impl RectanglesLocation {
    fn apply(&mut self, variables: &VariableValues<f64>) -> SpaceBaseArea<f64> {
        match self {
            RectanglesLocation::Area(ref mut a ,s,edge) => {
                s.apply( a,variables);
                let out = if let Some(edge) = edge {
                    a.hollow_edge(&edge)
                } else {
                    a.clone()
                };
                out
            }
            RectanglesLocation::Sized(near,s,x,y) => {
                let mut far = near.clone();
                far.fold_tangent(x,|v,z| { *v += z; });
                far.fold_normal(y,|v,z| { *v += z; });
                s.apply(&mut far,variables);
                SpaceBaseArea::new(near.clone(),far)
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            RectanglesLocation::Area(a,_,_) => a.len(),
            RectanglesLocation::Sized(a,_,_,_) => a.len()
        }
    }
}

pub(crate) struct Rectangles {
    elements: ProcessStanzaElements,
    program: TrianglesProgramLink,
    location: RectanglesLocation,
    allotments: Vec<Allotment>,
    left: f64,
    width: Option<f64>,
    kind: TrianglesKind  
}

impl Rectangles {
    pub(crate) fn new_area(layer: &mut Layer, geometry_yielder: &mut TrackTrianglesYielder, patina_yielder: &mut dyn PatinaYielder, area: &HoleySpaceBaseArea, allotments: &[Allotment], left: f64, hollow: bool, kind: &TrianglesKind, edge: &Option<HollowEdge<f64>>)-> Result<Rectangles,Message> {
        let (area,subs) = area.extract();
        let location = RectanglesLocation::Area(area,subs,edge.clone());
        Rectangles::real_new(layer,geometry_yielder,patina_yielder,location,allotments,left,hollow,kind)
    }

    pub(crate) fn new_sized(layer: &mut Layer, geometry_yielder: &mut TrackTrianglesYielder, patina_yielder: &mut dyn PatinaYielder, points: &HoleySpaceBase, x_sizes: Vec<f64>, y_sizes: Vec<f64>, allotments: &[Allotment], left: f64, hollow: bool, kind: &TrianglesKind)-> Result<Rectangles,Message> {
        let (points,subs) = points.extract();
        let location = RectanglesLocation::Sized(points,subs,x_sizes,y_sizes);
        Rectangles::real_new(layer,geometry_yielder,patina_yielder,location,allotments,left,hollow,kind)
    }

    fn real_new(layer: &mut Layer, geometry_yielder: &mut TrackTrianglesYielder, patina_yielder: &mut dyn PatinaYielder, location: RectanglesLocation, allotments: &[Allotment], left: f64, hollow: bool, kind: &TrianglesKind)-> Result<Rectangles,Message> {
        let builder = layer.draw(geometry_yielder,patina_yielder)?.get_process_mut();
        let indexes = if hollow {
            vec![0,1,2, 1,2,3, 2,3,4, 3,4,5, 4,5,6, 5,6,7, 6,7,0, 7,0,1]
        } else {
            vec![0,3,1,2,0,3]
        };
        let elements = builder.get_stanza_builder().make_elements(location.len(),&indexes)?;
        let mut out = Rectangles {
            elements, left,
            width: if hollow { Some(1.) } else { None },
            program: geometry_yielder.program()?.clone(),
            location,
            kind: kind.clone(),
            allotments: allotments.to_vec()
        };
        out.recompute(&VariableValues::new())?;
        Ok(out)
    }

    pub(crate) fn elements_mut(&mut self) -> &mut ProcessStanzaElements { &mut self.elements }
}

impl DynamicShape for Rectangles {
    fn recompute(&mut self, variables: &VariableValues<f64>) -> Result<(),Message> {
        let area = self.location.apply(variables);
        let (base,delta) = self.kind.add_spacebase_area(&area,&self.allotments,self.left,self.width);
        self.program.add_data(&mut self.elements,base,delta)?;
        if self.program.origin_base.is_some() || self.program.origin_delta.is_some() {
            let (origin_base,origin_delta) = self.kind.add_spacebase(&area.middle_base(),&self.allotments,self.left,self.width);
            self.program.add_origin_data(&mut self.elements,origin_base,origin_delta)?;
        }
        Ok(())
    }
}
