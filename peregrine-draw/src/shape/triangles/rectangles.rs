use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::geometry::{GeometryYielder, GeometryAdder };
use crate::shape::layers::layer::Layer;
use crate::shape::layers::patina::PatinaYielder;
use crate::shape::util::arrayutil::rectangle64;
use crate::webgl::{ ProcessStanzaElements };
use peregrine_data::{Allotment, CoordinateSystem, Flattenable, HoleySpaceBase, HoleySpaceBaseArea, HollowEdge, SpaceBase, SpaceBaseArea, SpaceBaseAreaParameterLocation, SpaceBaseParameterLocation, Substitutions, VariableValues};
use super::drawgroup::DrawGroup;
use super::triangleadder::TriangleAdder;
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
    program: TriangleAdder,
    location: RectanglesLocation,
    allotments: Vec<Allotment>,
    left: f64,
    width: Option<f64>,
    kind: DrawGroup
}

impl Rectangles {
    pub(crate) fn new_area(layer: &mut Layer, geometry_yielder: &mut GeometryYielder, patina_yielder: &mut dyn PatinaYielder, area: &HoleySpaceBaseArea, allotments: &[Allotment], left: f64, hollow: bool, kind: &DrawGroup, edge: &Option<HollowEdge<f64>>)-> Result<Rectangles,Message> {
        let (area,subs) = area.extract();
        let location = RectanglesLocation::Area(area,subs,edge.clone());
        Rectangles::real_new(layer,geometry_yielder,patina_yielder,location,allotments,left,hollow,kind)
    }

    pub(crate) fn new_sized(layer: &mut Layer, geometry_yielder: &mut GeometryYielder, patina_yielder: &mut dyn PatinaYielder, points: &HoleySpaceBase, x_sizes: Vec<f64>, y_sizes: Vec<f64>, allotments: &[Allotment], left: f64, hollow: bool, kind: &DrawGroup)-> Result<Rectangles,Message> {
        let (points,subs) = points.extract();
        let location = RectanglesLocation::Sized(points,subs,x_sizes,y_sizes);
        Rectangles::real_new(layer,geometry_yielder,patina_yielder,location,allotments,left,hollow,kind)
    }

    fn real_new(layer: &mut Layer, geometry_yielder: &mut GeometryYielder, patina_yielder: &mut dyn PatinaYielder, location: RectanglesLocation, allotments: &[Allotment], left: f64, hollow: bool, kind: &DrawGroup)-> Result<Rectangles,Message> {
        let builder = layer.get_process_builder(geometry_yielder,patina_yielder)?;
        let indexes = if hollow {
            vec![0,1,2, 1,2,3, 2,3,4, 3,4,5, 4,5,6, 5,6,7, 6,7,0, 7,0,1]
        } else {
            vec![0,3,1,2,0,3]
        };
        let elements = builder.get_stanza_builder().make_elements(location.len(),&indexes)?;
        let adder = match geometry_yielder.get_adder::<GeometryAdder>()? {
            GeometryAdder::Triangles(adder) => { adder },
            _ => { return Err(Message::CodeInvariantFailed(format!("bad adder"))) }
        };
        let mut out = Rectangles {
            elements, left,
            width: if hollow { Some(1.) } else { None },
            program: adder.clone(),
            location,
            kind: kind.clone(),
            allotments: allotments.to_vec()
        };
        out.recompute(&VariableValues::new())?;
        Ok(out)
    }

    pub(crate) fn elements_mut(&mut self) -> &mut ProcessStanzaElements { &mut self.elements }
}

fn add_spacebase(point: &SpaceBase<f64>, coord_system: &CoordinateSystem, allotments: &[Allotment], left: f64, width: Option<f64>) -> (Vec<f32>,Vec<f32>) {
    let area = SpaceBaseArea::new(point.clone(),point.clone());
    add_spacebase_area(&area,coord_system,allotments,left,width)
}

fn add_spacebase_area(area: &SpaceBaseArea<f64>, coord_system: &CoordinateSystem, allotments: &[Allotment], left: f64, width: Option<f64>)-> (Vec<f32>,Vec<f32>) {
    let mut base = vec![];
    let mut delta = vec![];
    let base_width = if width.is_some() { Some(0.) } else { None };
    let applied_left = if coord_system.is_tracking() { left } else { 0. };
    for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
        let top_left = allotment.transform_spacebase(&top_left);
        let bottom_right = allotment.transform_spacebase(&bottom_right);
        let (mut x0,mut y0,mut x1,mut y1) = (top_left.tangent,top_left.normal,bottom_right.tangent,bottom_right.normal);
        let (mut bx0,mut by0,mut bx1,mut by1) = (top_left.base-applied_left,0.,bottom_right.base-applied_left,0.);
        if !coord_system.is_tracking() {
            if x0 < 0. { x0 = -x0-1.; bx0 = 1.; }
            if x1 < 0. { x1 = -x1-1.; bx1 = 1.; }
        }
        if y0 < 0. { y0 = -y0-1.; by0 = 1.; }
        if y1 < 0. { y1 = -y1-1.; by1 = 1.; }
        rectangle64(&mut base, bx0,by0, bx1,by1,base_width);
        rectangle64(&mut delta, x0,y0,x1,y1,width);
    }
    (base,delta)
}

impl DynamicShape for Rectangles {
    fn recompute(&mut self, variables: &VariableValues<f64>) -> Result<(),Message> {
        let area = self.location.apply(variables);
        let (base,delta) = add_spacebase_area(&area,&self.kind.coord_system(),&self.allotments,self.left,self.width);
        self.program.add_data(&mut self.elements,base,delta,self.kind.depth())?;
        if self.program.origin_base.is_some() || self.program.origin_delta.is_some() {
            let (origin_base,origin_delta) = add_spacebase(&area.middle_base(),&self.kind.coord_system(),&self.allotments,self.left,self.width);
            self.program.add_origin_data(&mut self.elements,origin_base,origin_delta)?;
        }
        Ok(())
    }
}
