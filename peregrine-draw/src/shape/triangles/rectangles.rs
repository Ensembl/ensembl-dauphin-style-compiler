use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::geometry::{GeometryYielder, GeometryAdder };
use crate::shape::layers::layer::Layer;
use crate::shape::layers::patina::PatinaYielder;
use crate::shape::util::arrayutil::{rectangle4};
use crate::shape::util::iterators::eoe_throw;
use crate::webgl::{ ProcessStanzaElements };
use peregrine_data::{Allotment, CoordinateSystem, EachOrEvery, Flattenable, HoleySpaceBase, HoleySpaceBaseArea, HollowEdge, SpaceBase, SpaceBaseArea, SpaceBaseAreaParameterLocation, SpaceBaseParameterLocation, Substitutions, VariableValues};
use super::drawgroup::DrawGroup;
use super::triangleadder::TriangleAdder;
use crate::util::message::Message;

enum RectanglesLocation {
    Area(SpaceBaseArea<f64>,Substitutions<SpaceBaseAreaParameterLocation>,Option<HollowEdge<f64>>),
    Sized(SpaceBase<f64>,Substitutions<SpaceBaseParameterLocation>,Vec<f64>,Vec<f64>)
}

impl RectanglesLocation {
    fn any_dynamic(&self) -> bool {
        match self {
            RectanglesLocation::Area(_,s,_) => s.len() != 0,
            RectanglesLocation::Sized(_,s,_,_) => s.len() != 0
        }
    }

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
    allotments: EachOrEvery<Allotment>,
    left: f64,
    width: Option<f64>,
    kind: DrawGroup
}

impl Rectangles {
    pub(crate) fn new_area(layer: &mut Layer, geometry_yielder: &mut GeometryYielder, patina_yielder: &mut dyn PatinaYielder, area: &HoleySpaceBaseArea, allotments: &EachOrEvery<Allotment>, left: f64, hollow: bool, kind: &DrawGroup, edge: &Option<HollowEdge<f64>>)-> Result<Rectangles,Message> {
        let (area,subs) = area.extract();
        let location = RectanglesLocation::Area(area,subs,edge.clone());
        Rectangles::real_new(layer,geometry_yielder,patina_yielder,location,allotments,left,hollow,kind)
    }

    pub(crate) fn new_sized(layer: &mut Layer, geometry_yielder: &mut GeometryYielder, patina_yielder: &mut dyn PatinaYielder, points: &HoleySpaceBase, x_sizes: Vec<f64>, y_sizes: Vec<f64>, allotments: &EachOrEvery<Allotment>, left: f64, hollow: bool, kind: &DrawGroup)-> Result<Rectangles,Message> {
        let (points,subs) = points.extract();
        let location = RectanglesLocation::Sized(points,subs,x_sizes,y_sizes);
        Rectangles::real_new(layer,geometry_yielder,patina_yielder,location,allotments,left,hollow,kind)
    }

    fn real_new(layer: &mut Layer, geometry_yielder: &mut GeometryYielder, patina_yielder: &mut dyn PatinaYielder, location: RectanglesLocation, allotments: &EachOrEvery<Allotment>, left: f64, hollow: bool, kind: &DrawGroup)-> Result<Rectangles,Message> {
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
            allotments: allotments.clone()
        };
        out.recompute(&VariableValues::new())?;
        Ok(out)
    }

    pub(crate) fn elements_mut(&mut self) -> &mut ProcessStanzaElements { &mut self.elements }
}

fn add_spacebase4(point: &SpaceBase<f64>, coord_system: &CoordinateSystem, allotments: &EachOrEvery<Allotment>, left: f64, width: Option<f64>, depth: f64) -> Result<Vec<f32>,Message> {
    let area = SpaceBaseArea::new(point.clone(),point.clone());
    add_spacebase_area4(&area,coord_system,allotments,left,width,depth)
}

fn add_spacebase_area4(area: &SpaceBaseArea<f64>, coord_system: &CoordinateSystem, allotments: &EachOrEvery<Allotment>, left: f64, width: Option<f64>,depth: f64)-> Result<Vec<f32>,Message> {
    let mut data = vec![];
    let applied_left = if coord_system.is_tracking() { left } else { 0. };
    for ((top_left,bottom_right),allotment) in area.iter().zip(eoe_throw("sba1",allotments.iter(area.len()))?) {
        let top_left = allotment.transform_spacebase(&top_left);
        let bottom_right = allotment.transform_spacebase(&bottom_right);
        let (t_0,t_1,mut n_0,mut n_1) = (top_left.tangent,bottom_right.tangent,top_left.normal,bottom_right.normal);
        let (mut b_0,mut b_1) = (top_left.base,bottom_right.base);
        /* 
         * All coordinate systems have a principal direction. This is almost always horizontal however for things drawn
         * "sideways" (eg blanking boxes at left and right) it is vertical.
         * 
         * In the principal direction there is a "base" coordinate. For tracking co-ordinate systems this is the base-
         * pair position. For non-tracking coordinate systems, it is zero at the left and one at the right. There is
         * also a "tangent" co-ordinate in the principal direction which is measured in pixels and added on. This is to
         * allow non-scaling itmes (eg labels) and to ensure minimum sizes for things that would be very small.
         * 
         * In the non-principal direction there is the "normal" co-ordinate. In non-tracking co-ordinate systems, t
         * his wraps to -1 at the bottom, -2 is one pixel above that, and so on. So a line from zero to minus-one is 
         * from the top to the bottom.
         * 
         * Some non-tracking co-ordinate systems are "negative". These implicitly negate the normal co-ordinate, so that
         * top-is-bottom and bottom-is-top, for exmaple making the top and bottom ruler exact copies.
         * 
         * There are two formats for webgl, packed and unpacked. We use (x,y) and (z,a) as pairs. x,y are always
         * scaled per pixel. z is base co-oridnate or proportion-of-screen in the x direction, depending if
         * tracking or not. a is depth in the packed format, proportion of screen in the y direction if not.
         */
        /* Make tracking and non-tracking equivalent by subtracting bp centre. */
        if coord_system.is_tracking() {
            b_0 -= applied_left;
            b_1 -= applied_left;
        }
        if coord_system.packed_format() {
            /* We're packed, so that's enough. No negaite co-ordinate nonsense allowed for us. */
            rectangle4(&mut data, 
                t_0,n_0, t_1,n_1,
                b_0,depth,b_1,depth,
                width);
        } else {
            /* We're unpacked. f_0 and f_1 are "proportion of screenfuls to add", 1 for -ve, 0 for +ve */
            let (mut f_0,mut f_1) = (0.,0.);
            /* whole coord-system is negative, flip it */
            if coord_system.negative_pixels() {
                let (a,b) = (n_0,n_1);
                n_0 = -b-1.;
                n_1 = -a-1.;
            }
            /* negative values indicate "from end" so fix 1px difference (-1 is first rev. pixel and set f appropriately) */
            if n_0 < 0. { n_0 += 1.; f_0 = 1.; }
            if n_1 < 0. { n_1 += 1.; f_1 = 1.; }
            /* maybe flip x&y if sideways, either way draw it. */
            if coord_system.flip_xy() {
                rectangle4(&mut data, n_0,t_0, n_1,t_1,f_0,b_0,f_1,b_1,width);
            } else {
                rectangle4(&mut data, t_0,n_0, t_1,n_1,b_0,f_0,b_1,f_1,width);
            }
        }
    }
    Ok(data)
}

impl DynamicShape for Rectangles {
    fn any_dynamic(&self) -> bool {
        self.location.any_dynamic()
    }

    fn recompute(&mut self, variables: &VariableValues<f64>) -> Result<(),Message> {
        let area = self.location.apply(variables);
        let gl_depth = 1.0 - (self.kind.depth() as f64+128.) / 255.;
        let data = add_spacebase_area4(&area,&self.kind.coord_system(),&self.allotments,self.left,self.width,gl_depth)?;
        self.program.add_data4(&mut self.elements,data,self.kind.depth())?;
        if self.program.origin_coords.is_some() {
            let data= add_spacebase4(&area.middle_base(),&self.kind.coord_system(),&self.allotments,self.left,self.width,gl_depth)?;
            self.program.add_origin_data4(&mut self.elements,data)?;
        }
        Ok(())
    }
}
