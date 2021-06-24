use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::layer::Layer;
use crate::shape::layers::patina::PatinaYielder;
use crate::webgl::{AttribHandle, ProcessStanzaAddable, ProcessStanzaElements, ProgramBuilder};
use peregrine_data::{
    Allotment, AllotmentPosition, Flattenable, HoleySpaceBase, HoleySpaceBaseArea, HollowEdge, PositionVariant,
    SpaceBase, SpaceBaseArea, SpaceBaseAreaParameterLocation, SpaceBaseParameterLocation, Substitutions, VariableValues
};
use super::super::util::arrayutil::rectangle64;
use crate::shape::layers::geometry::{GeometryProcessName, GeometryProgram, GeometryProgramName, GeometryYielder};
use crate::util::message::Message;

fn flip(allotment: &Allotment) -> f64 {
    match  match allotment.position() {
        AllotmentPosition::BaseLabel(p,_) => p,
        AllotmentPosition::SpaceLabel(p,_) => p,
        _ => &PositionVariant::HighPriority
    } {
        PositionVariant::HighPriority => 1.,
        PositionVariant::LowPriority => -1.
    }
}

pub(crate) struct TrackTrianglesYielder {
    geometry_process_name: GeometryProcessName,
    track_triangles: Option<TrackTrianglesProgram>
}

impl<'a> GeometryYielder for TrackTrianglesYielder {
    fn name(&self) -> &GeometryProcessName { &self.geometry_process_name }

    fn make(&mut self, builder: &ProgramBuilder) -> Result<GeometryProgram,Message> {
        self.geometry_process_name.get_program_name().make_geometry_program(builder)
    }

    fn set(&mut self, program: &GeometryProgram) -> Result<(),Message> {
        self.track_triangles = Some(match program {
            GeometryProgram::BaseLabelTriangles(t) => t,
            GeometryProgram::SpaceLabelTriangles(t) => t,
            GeometryProgram::TrackTriangles(t) => t,
            GeometryProgram::WindowTriangles(t) => t,
            _ => { Err(Message::CodeInvariantFailed(format!("mismatched program: tracktriangles")))? }
        }.clone());
        Ok(())
    }
}

impl TrackTrianglesYielder {
    pub(crate) fn new(geometry_process_name: &GeometryProcessName) -> TrackTrianglesYielder {
        TrackTrianglesYielder {
            geometry_process_name: geometry_process_name.clone(),
            track_triangles: None
        }
    }

    pub(crate) fn track_triangles(&self) -> Result<&TrackTrianglesProgram,Message> {
        self.track_triangles.as_ref().ok_or_else(|| Message::CodeInvariantFailed(format!("using accessor without setting")))
    }
}

#[derive(Debug,Clone)]
pub enum TrianglesKind {
    Track,
    Base,
    Space,
    Window(i64)
}

impl TrianglesKind {
    fn add_spacebase(&self, point: &SpaceBase<f64>, allotments: &[Allotment], left: f64, width: Option<f64>) -> (Vec<f32>,Vec<f32>) {
        let area = SpaceBaseArea::new(point.clone(),point.clone());
        self.add_spacebase_area(&area,allotments,left,width)
    }

    fn add_spacebase_area(&self, area: &SpaceBaseArea<f64>, allotments: &[Allotment], left: f64, width: Option<f64>)-> (Vec<f32>,Vec<f32>) {
        let mut base = vec![];
        let mut delta = vec![];
        let base_width = if width.is_some() { Some(0.) } else { None };
        match self {
            TrianglesKind::Track => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let base_y = allotment.position().offset() as f64;
                    rectangle64(&mut base, *top_left.base-left, base_y, *bottom_right.base-left,base_y,base_width);
                    rectangle64(&mut delta, *top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal,width);
                }
            },
            TrianglesKind::Base => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let flip_y = flip(allotment);
                    rectangle64(&mut base, *top_left.base-left, flip_y, *bottom_right.base-left,flip_y,base_width);
                    rectangle64(&mut delta, *top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal,width);
                }        
            },
            TrianglesKind::Space => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let flip_x = flip(allotment);
                    let base_y = allotment.position().offset() as f64;
                    rectangle64(&mut base, flip_x, base_y, flip_x,base_y,base_width);
                    rectangle64(&mut delta, *top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal,width);
                }
            },
            TrianglesKind::Window(_) => {
                for ((top_left,bottom_right),_) in area.iter().zip(allotments.iter().cycle()) {
                    let (mut x0,mut y0,mut x1,mut y1) = (*top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal);
                    let (mut bx0,mut by0,mut bx1,mut by1) = (0.,0.,0.,0.);
                    if x0 < 0. { x0 = -x0-1.; bx0 = 1.; }
                    if y0 < 0. { y0 = -y0-1.; by0 = 1.; }
                    if x1 < 0. { x1 = -x1-1.; bx1 = 1.; }
                    if y1 < 0. { y1 = -y1-1.; by1 = 1.; }
                    rectangle64(&mut base, bx0,by0, bx1,by1,base_width);
                    rectangle64(&mut delta, x0,y0,x1,y1,width);
                }
            }
        }
        (base,delta)
    }

    pub(crate) fn geometry_process_name(&self) -> GeometryProcessName {
        let (program,priority) = match self {
            TrianglesKind::Track => (GeometryProgramName::TrackTriangles,0),
            TrianglesKind::Base => (GeometryProgramName::BaseLabelTriangles,0),
            TrianglesKind::Space => (GeometryProgramName::SpaceLabelTriangles,0),
            TrianglesKind::Window(p) => (GeometryProgramName::WindowTriangles,*p)
        };
        GeometryProcessName::new(program,priority)
    }

    pub(crate) fn geometry_yielder(&self) -> TrackTrianglesYielder {
        TrackTrianglesYielder::new(&self.geometry_process_name())
    }
}

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
    program: TrackTrianglesProgram,
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
            program: geometry_yielder.track_triangles()?.clone(),
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
        let mut origin_base = None;
        let mut origin_delta = None;
        if self.program.origin_base.is_some() || self.program.origin_delta.is_some() {
            let (a,b) = self.kind.add_spacebase(&area.middle_base(),&self.allotments,self.left,self.width);
            origin_base = Some(a);
            origin_delta = Some(b);
        }
        self.elements.add(&self.program.delta,delta,2)?;
        self.elements.add(&self.program.base,base,2)?;
        if let Some(origin_base_handle) = &self.program.origin_base {
            self.elements.add(origin_base_handle,origin_base.unwrap(),2)?;
        }
        if let Some(origin_delta_handle) = &self.program.origin_delta {
            self.elements.add(origin_delta_handle,origin_delta.unwrap(),2)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct TrackTrianglesProgram {
    base: AttribHandle,
    delta: AttribHandle,
    origin_base: Option<AttribHandle>,
    origin_delta: Option<AttribHandle>,
}

impl TrackTrianglesProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<TrackTrianglesProgram,Message> {
        Ok(TrackTrianglesProgram {
            base: builder.get_attrib_handle("aBase")?,
            delta: builder.get_attrib_handle("aDelta")?,
            origin_base: builder.try_get_attrib_handle("aOriginBase"),
            origin_delta: builder.try_get_attrib_handle("aOriginDelta")
        })
    }
}
