use std::sync::{Arc, Mutex};

use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::geometry::{GeometryYielder, GeometryAdder };
use crate::shape::layers::layer::Layer;
use crate::shape::layers::patina::PatinaYielder;
use crate::shape::util::arrayutil::{rectangle4};
use crate::shape::util::iterators::eoe_throw;
use crate::webgl::{ ProcessStanzaElements };
use peregrine_data::reactive::{Observable, Observer};
use peregrine_data::{ SpaceBaseArea, SpaceBase, PartialSpaceBase, HollowEdge2, SpaceBasePoint, LeafStyle };
use peregrine_toolkit::eachorevery::EachOrEvery;
use peregrine_toolkit::lock;
use super::drawgroup::DrawGroup;
use super::triangleadder::TriangleAdder;
use crate::util::message::Message;

fn apply_wobble(pos: &SpaceBase<f64,LeafStyle>, wobble: &SpaceBase<Observable<'static,f64>,()>) -> SpaceBase<f64,LeafStyle> {
    let wobble = wobble.map_all(|obs| obs.get());
    pos.merge(wobble,SpaceBasePoint {
        base: &|a,b| { *a+*b },
        normal: &|a,b| { *a+*b },
        tangent: &|a,b| { *a+*b },
        allotment: &|a,_| { a.clone() }
    })
}

#[cfg_attr(debug_assertions,derive(Debug))]
struct RectanglesLocationArea {
    spacebase: SpaceBaseArea<f64,LeafStyle>,
    wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>,
    wobbled_spacebase: Arc<Mutex<SpaceBaseArea<f64,LeafStyle>>>,
    depth: EachOrEvery<i8>,
    edge: Option<HollowEdge2<f64>>
}

impl RectanglesLocationArea {
    fn new(spacebase: &SpaceBaseArea<f64,LeafStyle>, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>, depth: EachOrEvery<i8>, edge: Option<HollowEdge2<f64>>) -> Result<RectanglesLocationArea,Message> {
        Ok(RectanglesLocationArea {
            wobbled_spacebase: Arc::new(Mutex::new(area_to_rectangle(spacebase,&wobble,&edge)?)),
            spacebase: spacebase.clone(),
            wobble, depth, edge,
        })
    }

    fn depths(&self) -> &EachOrEvery<i8> { &self.depth }
    fn len(&self) -> usize { self.spacebase.len() }
    fn any_dynamic(&self) -> bool { self.wobble.is_some() }
    fn wobbled_location(&self) -> SpaceBaseArea<f64,LeafStyle> { lock!(self.wobbled_spacebase).clone() }

    fn wobble(&mut self) -> Option<Box<dyn FnMut() + 'static>> {
        self.wobble.as_ref().map(|wobble| {
            let wobble = wobble.clone();
            let area = self.spacebase.clone();
            let wobbled = self.wobbled_spacebase.clone();
            let edge = self.edge.clone();
            Box::new(move || {
                if let Ok(area) = area_to_rectangle(&area,&Some(wobble.clone()),&edge) {
                    *lock!(wobbled) = area;
                }
            }) as Box<dyn FnMut() + 'static>
        })
    }

    fn watch(&self, observer: &mut Observer<'static>) {
        if let Some(wobble) = &self.wobble {
            for obs in wobble.top_left().iter() {
                observer.observe(obs.base);
                observer.observe(obs.normal);
                observer.observe(obs.tangent);
            }
            for obs in wobble.bottom_right().iter() {
                observer.observe(obs.base);
                observer.observe(obs.normal);
                observer.observe(obs.tangent);
            }
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
struct RectanglesLocationSized {
    spacebase: SpaceBase<f64,LeafStyle>,
    wobble: Option<SpaceBase<Observable<'static,f64>,()>>,
    wobbled_spacebase: Arc<Mutex<SpaceBaseArea<f64,LeafStyle>>>,
    depth: EachOrEvery<i8>,
    size_x: Vec<f64>,
    size_y: Vec<f64>
}

fn sized_to_rectangle(spacebase: &SpaceBase<f64,LeafStyle>, wobble: &Option<SpaceBase<Observable<'static,f64>,()>>, size_x: &[f64], size_y: &[f64]) -> Result<SpaceBaseArea<f64,LeafStyle>,Message> {
    let wobbled = if let Some(wobble) = wobble {
        apply_wobble(spacebase,&wobble)
    } else {
        spacebase.clone()
    };
    let mut far = wobbled.clone();
    far.fold_tangent(size_x,|v,z| { *v + z });
    far.fold_normal(size_y,|v,z| { *v + z });
    let area = eoe_throw("rl1",SpaceBaseArea::new(
        PartialSpaceBase::from_spacebase(spacebase.clone()),
                PartialSpaceBase::from_spacebase(far)))?;
    Ok(area)
}

fn apply_hollow(area: &SpaceBaseArea<f64,LeafStyle>, edge: &Option<HollowEdge2<f64>>) -> SpaceBaseArea<f64,LeafStyle> {
    if let Some(edge) = edge {
        area.hollow_edge(&edge)
    } else {
        area.clone()
    }
}

fn area_to_rectangle(area: &SpaceBaseArea<f64,LeafStyle>,  wobble: &Option<SpaceBaseArea<Observable<'static,f64>,()>>, edge: &Option<HollowEdge2<f64>>) -> Result<SpaceBaseArea<f64,LeafStyle>,Message> {
    if let Some(wobble) = wobble {
        let top_left = apply_wobble(area.top_left(),wobble.top_left());
        let bottom_right = apply_wobble(area.bottom_right(),wobble.bottom_right());
        let wobbled = SpaceBaseArea::new(PartialSpaceBase::from_spacebase(top_left),PartialSpaceBase::from_spacebase(bottom_right));
        if let Some(wobbled) = wobbled {
            return Ok(apply_hollow(&wobbled,edge));
        }
    }
    Ok(apply_hollow(area,edge))
}

impl RectanglesLocationSized {
    fn new(spacebase: &SpaceBase<f64,LeafStyle>, wobble: Option<SpaceBase<Observable<'static,f64>,()>>, depth: EachOrEvery<i8>, size_x: Vec<f64>, size_y: Vec<f64>) -> Result<RectanglesLocationSized,Message> {
        Ok(RectanglesLocationSized { 
            wobbled_spacebase: Arc::new(Mutex::new(sized_to_rectangle(spacebase,&wobble,&size_x,&size_y)?)),
            spacebase: spacebase.clone(),
            wobble, depth, size_x, size_y
        })
    }

    fn depths(&self) -> &EachOrEvery<i8> { &self.depth }
    fn len(&self) -> usize { self.spacebase.len() }
    fn any_dynamic(&self) -> bool { self.wobble.is_some() }
    fn wobbled_location(&self) -> SpaceBaseArea<f64,LeafStyle> { lock!(self.wobbled_spacebase).clone() }

    fn wobble(&mut self) -> Option<Box<dyn FnMut() + 'static>> {
        self.wobble.as_ref().map(|wobble| {
            let wobble = wobble.clone();
            let pos = self.spacebase.clone();
            let wobbled = self.wobbled_spacebase.clone();
            let size_x = self.size_x.clone();
            let size_y = self.size_y.clone();
            Box::new(move || {
                if let Ok(sized) =  sized_to_rectangle(&pos,&Some(wobble.clone()),&size_x,&size_y) {
                    *lock!(wobbled) = sized;
                }
            }) as Box<dyn FnMut() + 'static>
        })
    }

    // XXX PartialEq + Hash for collision
    fn watch(&self, observer: &mut Observer<'static>) {
        if let Some(wobble) = &self.wobble {
            for obs in wobble.iter() {
                observer.observe(obs.base);
                observer.observe(obs.normal);
                observer.observe(obs.tangent);
            }
        }
    }    
}

#[cfg_attr(debug_assertions,derive(Debug))]
enum RectanglesLocation {
    Area(RectanglesLocationArea),
    Sized(RectanglesLocationSized)
}

impl RectanglesLocation {
    fn wobbled_location(&self) -> SpaceBaseArea<f64,LeafStyle> {
        match self {
            RectanglesLocation::Area(area) => area.wobbled_location(),
            RectanglesLocation::Sized(sized) => sized.wobbled_location()
        }
    }

    fn depths(&self) -> &EachOrEvery<i8> {
        match self {
            RectanglesLocation::Area(area) => area.depths(),
            RectanglesLocation::Sized(sized) => sized.depths()
        }
    }

    fn any_dynamic(&self) -> bool {
        match self {
            RectanglesLocation::Area(area) => area.any_dynamic(),
            RectanglesLocation::Sized(sized) => sized.any_dynamic()
        }
    }

    fn wobble(&mut self) -> Option<Box<dyn FnMut() + 'static>> {
        match self {
            RectanglesLocation::Area(area) => area.wobble(),
            RectanglesLocation::Sized(sized) => sized.wobble()
        }
    }

    // XXX PartialEq + Hash for collision
    fn watch(&self, observer: &mut Observer<'static>) {
        match self {
            RectanglesLocation::Area(area) => area.watch(observer),
            RectanglesLocation::Sized(sized) => sized.watch(observer)
        }
    } 

    fn len(&self) -> usize {
        match self {
            RectanglesLocation::Area(area) => area.len(),
            RectanglesLocation::Sized(sized) => sized.len()
        }
    }
}

pub(crate) struct RectanglesData {
    elements: ProcessStanzaElements,
    program: TriangleAdder,
    location: RectanglesLocation,
    left: f64,
    width: Option<f64>,
    kind: DrawGroup
}

impl RectanglesData {
    pub(crate) fn new_area(layer: &mut Layer, geometry_yielder: &mut GeometryYielder, patina_yielder: &mut dyn PatinaYielder, area: &SpaceBaseArea<f64,LeafStyle>, depth: &EachOrEvery<i8>, left: f64, hollow: bool, kind: &DrawGroup, edge: &Option<HollowEdge2<f64>>, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>)-> Result<RectanglesData,Message> {
        let location = RectanglesLocation::Area(RectanglesLocationArea::new(area,wobble,depth.clone(),edge.clone())?);
        Self::real_new(layer,geometry_yielder,patina_yielder,location,left,hollow,kind)
    }

    pub(crate) fn new_sized(layer: &mut Layer, geometry_yielder: &mut GeometryYielder, patina_yielder: &mut dyn PatinaYielder, points: &SpaceBase<f64,LeafStyle>, x_sizes: Vec<f64>, y_sizes: Vec<f64>, depth: &EachOrEvery<i8>, left: f64, hollow: bool, kind: &DrawGroup, wobble: Option<SpaceBase<Observable<'static,f64>,()>>)-> Result<RectanglesData,Message> {
        let location = RectanglesLocation::Sized(RectanglesLocationSized::new(points,wobble,depth.clone(),x_sizes,y_sizes)?);
        Self::real_new(layer,geometry_yielder,patina_yielder,location,left,hollow,kind)
    }

    fn real_new(layer: &mut Layer, geometry_yielder: &mut GeometryYielder, patina_yielder: &mut dyn PatinaYielder, location: RectanglesLocation, left: f64, hollow: bool, kind: &DrawGroup)-> Result<RectanglesData,Message> {
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
        Ok(RectanglesData {
            elements, left,
            width: if hollow { Some(1.) } else { None },
            program: adder.clone(),
            location,
            kind: kind.clone()
        })
    }

    pub(crate) fn elements_mut(&mut self) -> &mut ProcessStanzaElements { &mut self.elements }

    fn any_dynamic(&self) -> bool {
        self.location.any_dynamic()
    }

    fn recompute(&mut self) -> Result<(),Message> {
        let area = self.location.wobbled_location();
        let depth_in = self.location.depths();
        let (data,depth) = add_spacebase_area4(&area,&depth_in,&self.kind,self.left,self.width)?;
        self.program.add_data4(&mut self.elements,data,depth)?;
        if self.program.origin_coords.is_some() {
            let (data,_)= add_spacebase4(&PartialSpaceBase::from_spacebase(area.middle_base()),&depth_in,&self.kind,self.left,self.width)?;
            self.program.add_origin_data4(&mut self.elements,data)?;
        }
        Ok(())
    }
}

pub(crate) struct Rectangles {
    data: Arc<Mutex<RectanglesData>>,
    wobble: Option<Observer<'static>>
}

impl Rectangles {
    pub(crate) fn new(data: RectanglesData) -> Rectangles {
        let data = Arc::new(Mutex::new(data));
        let wobble_cb = lock!(data).location.wobble();
        let wobble = wobble_cb.map(|cb| Observer::new_boxed(cb));
        let mut out = Rectangles {
            data,
            wobble
        };
        if let Some(wobble) = &mut out.wobble {
            lock!(out.data).location.watch(wobble);
        }
        out.recompute();
        out
    }
}

fn add_spacebase4(point: &PartialSpaceBase<f64,LeafStyle>,depth: &EachOrEvery<i8>, group: &DrawGroup, left: f64, width: Option<f64>) -> Result<(Vec<f32>,Vec<f32>),Message> {
    let area = eoe_throw("as1",SpaceBaseArea::new(point.clone(),point.clone()))?;
    add_spacebase_area4(&area,depth,group,left,width)
}

fn add_spacebase_area4(area: &SpaceBaseArea<f64,LeafStyle>, depth: &EachOrEvery<i8>, group: &DrawGroup, left: f64, width: Option<f64>)-> Result<(Vec<f32>,Vec<f32>),Message> {
    let mut data = vec![];
    let mut depths = vec![];
    for ((top_left,bottom_right),depth) in area.iter().zip(eoe_throw("t",depth.iter(area.len()))?) {
        let (t_0,t_1,mut n_0,mut n_1) = (*top_left.tangent,*bottom_right.tangent,*top_left.normal,*bottom_right.normal);
        let (mut b_0,mut b_1) = (*top_left.base,*bottom_right.base);
        /* 
         * All coordinate systems have a principal direction. This is almost always horizontal however for things drawn
         * "sideways" (eg blanking boxes at left and right) it is vertical.
         * 
         * In the principal direction there is a "base" coordinate. For tracking co-ordinate systems this is the base-
         * pair position. For non-tracking coordinate systems, it is zero at the left and one at the right. There is
         * also a "tangent" co-ordinate in the principal direction which is measured in pixels and added on. This is to
         * allow non-scaling itmes (eg labels) and to ensure minimum sizes for things that would be very small.
         * 
         * In the non-principal direction there is the "normal" co-ordinate. Except for the compact Tracking coordinate
         * system (which is optimised for efficiency), co-ordinate systems, this value this wraps to -1 at the bottom, 
         * 2 is one pixel above that, and so on. So a line from zero to minus-one is from the top to the bottom.
         * 
         * Some non-tracking co-ordinate systems are "negative". These implicitly negate the normal co-ordinate, so that
         * top-is-bottom and bottom-is-top, for exmaple making the top and bottom ruler exact copies.
         * 
         * There are two formats for webgl, packed and unpacked. We use (x,y) and (z,a) as pairs. x,y are always
         * scaled per pixel. z is base co-oridnate or proportion-of-screen in the x direction, depending if
         * tracking or not. a is depth in the packed format, proportion of screen in the y direction if not.
         */
        /* Make tracking and non-tracking equivalent by subtracting bp centre. */
        if group.coord_system().is_tracking() {
            b_0 -= left;
            b_1 -= left;
        }
        let gl_depth = 1.0 - (*depth as f64+128.) / 255.;
        if group.packed_format() {
            /* We're packed, so that's enough. No negaite co-ordinate nonsense allowed for us. */
            rectangle4(&mut data, 
                t_0,n_0, t_1,n_1,
                b_0,gl_depth,b_1,gl_depth,
                width);
        } else {
            let num_points = if width.is_some() { 8 } else { 4 };
            for _ in 0..num_points {
                depths.push(gl_depth as f32);
            }
            /* We're unpacked. f_0 and f_1 are "proportion of screenfuls to add", 1 for -ve, 0 for +ve */
            let (mut f_0,mut f_1) = (0.,0.);
            /* whole coord-system is negative, flip it */
            if group.coord_system().negative_pixels() {
                let (a,b) = (n_0,n_1);
                n_0 = -b-1.;
                n_1 = -a-1.;
            }
            /* negative values indicate "from end" so fix 1px difference (-1 is first rev. pixel) and set f appropriately */
            if n_0 < 0. { n_0 += 1.; f_0 = 1.; }
            if n_1 < 0. { n_1 += 1.; f_1 = 1.; }
            /* maybe flip x&y if sideways, either way draw it. */
            if group.coord_system().flip_xy() {
                rectangle4(&mut data, n_0,t_0, n_1,t_1,f_0,b_0,f_1,b_1,width);
            } else {
                rectangle4(&mut data, t_0,n_0, t_1,n_1,b_0,f_0,b_1,f_1,width);
            }
        }
    }
    Ok((data,depths))
}

impl DynamicShape for Rectangles {
    fn any_dynamic(&self) -> bool {
        lock!(self.data).any_dynamic()
    }

    fn recompute(&mut self) -> Result<(),Message> {
        lock!(self.data).recompute()
    }
}
