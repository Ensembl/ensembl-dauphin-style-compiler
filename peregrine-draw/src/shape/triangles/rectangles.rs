use std::sync::{Arc, Mutex};
use crate::shape::core::drawshape::GLAttachmentPoint;
use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::geometry::{GeometryFactory, GeometryProcessName};
use crate::shape::util::arrayutil::rectangle4;
use crate::shape::util::eoethrow::{eoe_throw2};
use crate::webgl::global::WebGlGlobal;
use crate::webgl::{ ProcessStanzaElements, ProcessBuilder };
use eachorevery::EachOrEvery;
use peregrine_data::reactive::{Observable, Observer};
use peregrine_data::{ SpaceBaseArea, SpaceBase, PartialSpaceBase, HollowEdge2, SpaceBasePoint, AuxLeaf };
use peregrine_toolkit::error::{Error, err_web_drop};
use peregrine_toolkit::{lock};
use super::arearectangles::RectanglesLocationArea;
use super::drawgroup::DrawGroup;
use super::sizedrectangles::RectanglesLocationSized;
use super::triangleadder::TriangleAdder;

pub(super) trait RectanglesImpl {
    fn depths(&self) -> &EachOrEvery<i8>;
    fn any_dynamic(&self) -> bool;
    fn wobble(&mut self) -> Option<Box<dyn FnMut() + 'static>>;
    fn watch(&self, observer: &mut Observer<'static>);
    fn len(&self) -> usize;
    fn wobbled_location(&self) -> (SpaceBaseArea<f64,AuxLeaf>,Option<SpaceBase<f64,()>>);
}

pub(super) fn apply_wobble<A: Clone>(pos: &SpaceBase<f64,A>, wobble: &SpaceBase<Observable<'static,f64>,()>) -> SpaceBase<f64,A> {
    let wobble = wobble.map_all(|obs| obs.get());
    pos.merge(wobble,SpaceBasePoint {
        base: &|a,b| { *a+*b },
        normal: &|a,b| { *a+*b },
        tangent: &|a,b| { *a+*b },
        allotment: &|a,_| { a.clone() }
    })
}

pub(crate) struct RectanglesData {
    elements: ProcessStanzaElements,
    program: TriangleAdder,
    location: Box<dyn RectanglesImpl>,
    left: f64,
    width: Option<f64>,
    kind: DrawGroup
}

impl RectanglesData {
    fn new_area(builder: &mut ProcessBuilder, area: &SpaceBaseArea<f64,AuxLeaf>, run: &Option<SpaceBase<f64,()>>, depth: &EachOrEvery<i8>, left: f64, hollow: Option<f64>, kind: &DrawGroup, edge: &Option<HollowEdge2<f64>>, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>)-> Result<RectanglesData,Error> {
        let location = RectanglesLocationArea::new(area,run,wobble,depth.clone(),edge.clone())?;
        Self::real_new(builder,Box::new(location),left,hollow,kind)
    }

    fn new_sized(builder: &mut ProcessBuilder, points: &SpaceBase<f64,AuxLeaf>, run: &Option<SpaceBase<f64,()>>, x_sizes: Vec<f64>, y_sizes: Vec<f64>, depth: &EachOrEvery<i8>, left: f64, hollow: Option<f64>, kind: &DrawGroup, attachment: GLAttachmentPoint, wobble: Option<SpaceBase<Observable<'static,f64>,()>>)-> Result<RectanglesData,Error> {
        let location = RectanglesLocationSized::new(points,run,wobble,depth.clone(),x_sizes,y_sizes,attachment)?;
        Self::real_new(builder,Box::new(location),left,hollow,kind)
    }

    fn real_new(builder: &mut ProcessBuilder, location: Box<dyn RectanglesImpl>, left: f64, hollow: Option<f64>, kind: &DrawGroup)-> Result<RectanglesData,Error> {
        let adder = TriangleAdder::new(builder)?;
        let indexes = if hollow.is_some() {
            vec![0,1,2, 1,2,3, 2,3,4, 2,4,5, 4,5,6, 5,6,7, 6,7,0, 6,0,1]
        } else {
            vec![0,3,1, 2,0,3]
        };
        let elements = builder.get_stanza_builder().make_elements(location.len(),&indexes)?;
        Ok(RectanglesData {
            elements, left,
            width: hollow,
            program: adder.clone(),
            location,
            kind: kind.clone()
        })
    }

    pub(crate) fn elements_mut(&mut self) -> &mut ProcessStanzaElements { &mut self.elements }

    fn any_dynamic(&self) -> bool {
        self.location.any_dynamic()
    }

    fn recompute(&mut self, gl: &WebGlGlobal) -> Result<(),Error> {
        let (area,run) = self.location.wobbled_location();
        let depth_in = self.location.depths();
        let (data,depth) = add_spacebase_area4(&area,&depth_in,&self.kind,self.left,self.width,Some(gl.device_pixel_ratio()))?;
        self.program.add_data(&mut self.elements,data,depth)?;
        if self.program.origin_coords.is_some() {
            let (data,_)= add_spacebase4(&PartialSpaceBase::from_spacebase(area.top_left().clone()),&depth_in,&self.kind,self.left,self.width,None)?;
            self.program.add_origin_data(&mut self.elements,data)?;
        }
        if self.program.run_coords.is_some() {
            let run = run.unwrap_or(area.top_left().map_allotments(|_| ()));
            let (data,_)= add_spacebase4(&PartialSpaceBase::from_spacebase(run),&depth_in,&self.kind,self.left,self.width,None)?;
            self.program.add_run_data(&mut self.elements,data)?;
        }
        Ok(())
    }
}

pub(crate) struct RectanglesDataFactory {
    draw_group: DrawGroup
}

impl RectanglesDataFactory {
    pub(crate) fn new(draw_group: &DrawGroup) -> RectanglesDataFactory {
        RectanglesDataFactory {
            draw_group: draw_group.clone()
        }
    }

    pub(crate) fn make_area(&self, builder: &mut ProcessBuilder, area: &SpaceBaseArea<f64,AuxLeaf>, run: &Option<SpaceBase<f64,()>>, depth: &EachOrEvery<i8>, left: f64, hollow: Option<f64>, edge: &Option<HollowEdge2<f64>>, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>)-> Result<RectanglesData,Error> {
        RectanglesData::new_area(builder,area,run,depth,left,hollow,&self.draw_group,edge,wobble)
    }

    pub(crate) fn make_sized(&self, builder: &mut ProcessBuilder, points: &SpaceBase<f64,AuxLeaf>, run: &Option<SpaceBase<f64,()>>, x_sizes: Vec<f64>, y_sizes: Vec<f64>, depth: &EachOrEvery<i8>, left: f64, hollow: Option<f64>, attachment: GLAttachmentPoint, wobble: Option<SpaceBase<Observable<'static,f64>,()>>)-> Result<RectanglesData,Error> {
        RectanglesData::new_sized(builder,points,run,x_sizes,y_sizes,depth,left,hollow,&self.draw_group,attachment,wobble)
    }
}

impl GeometryFactory for RectanglesDataFactory {
    fn geometry_name(&self) -> GeometryProcessName {
        GeometryProcessName::Triangles(self.draw_group.geometry())
    }
}

pub(crate) struct Rectangles {
    data: Arc<Mutex<RectanglesData>>,
    wobble: Option<Observer<'static>>
}

impl Rectangles {
    pub(crate) fn new(data: RectanglesData, gl: &WebGlGlobal) -> Rectangles {
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
        err_web_drop(out.recompute(gl));
        out
    }
}

fn add_spacebase4<A: Clone>(point: &PartialSpaceBase<f64,A>,depth: &EachOrEvery<i8>, group: &DrawGroup, left: f64, width: Option<f64>, dpr: Option<f32>) -> Result<(Vec<f32>,Vec<f32>),Error> {
    let area = eoe_throw2("as1",SpaceBaseArea::new(point.clone(),point.clone()))?;
    add_spacebase_area4(&area,depth,group,left,width,dpr)
}


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


pub(super) fn fix_normal_unpacked(mut n: f64, group: &DrawGroup) -> (f64,f64) {
    /* We're unpacked. f_0 and f_1 are "proportion of screenfuls to add", 1 for -ve, 0 for +ve */
    let mut f = 0.;
    /* whole coord-system is negative, flip it */
    if group.coord_system().up_from_bottom() {
        n = -n-1.;
    }
    /* negative values indicate "from end" so fix 1px difference (-1 is first rev. pixel) and set f appropriately */
    if n < 0. { n += 1.; f = 1.; }
    (n,f)
}

fn add_spacebase_area4<A>(area: &SpaceBaseArea<f64,A>, depth: &EachOrEvery<i8>, group: &DrawGroup, left: f64, mut width: Option<f64>, dpr: Option<f32>)-> Result<(Vec<f32>,Vec<f32>),Error> {
    let mut data = vec![];
    let mut depths = vec![];
    if let Some(dpr) = dpr {
        width = width.map(|width| {
            if width == 0. { (1./dpr) as f64 } else { width }
        });
    }
    let mut coords = vec![];
    for ((top_left,bottom_right),depth) in area.iter().zip(eoe_throw2("t",depth.iter(area.len()))?) {
        let (t_0,t_1,n_0,mut n_1) = (*top_left.tangent,*bottom_right.tangent,*top_left.normal,*bottom_right.normal);
        if let Some(dpr) = dpr {
            if n_0 == n_1 && width.is_none() { /* finest lines. When hollow, fixed elsewhere */
                n_1 += (1./dpr) as f64;
            }    
        }
        let (mut b_0,mut b_1) = (*top_left.base,*bottom_right.base);
        /* Make tracking and non-tracking equivalent by subtracting bp centre. */
        if group.coord_system().is_tracking() {
            b_0 -= left;
            b_1 -= left;
        }
        let gl_depth = 1.0 - (*depth as f64+128.) / 255.;
        if group.packed_format() {
            /* We're packed, so that's enough. No negative co-ordinate nonsense allowed for us. */
            coords.push(((t_0,n_0,b_0,gl_depth),(t_1,n_1,b_1,gl_depth)));
        } else {
            let num_points = if width.is_some() { 8 } else { 4 };
            for _ in 0..num_points {
                depths.push(gl_depth as f32);
            }
            let (n_0,f_0) = fix_normal_unpacked(n_0,group);
            let (n_1,f_1) = fix_normal_unpacked(n_1,group);
            /* maybe flip x&y if sideways, either way draw it. */
            if group.coord_system().flip_xy() {
                coords.push(((n_0,t_0,f_0,b_0),(n_1,t_1,f_1,b_1)));
            } else {
                coords.push(((t_0,n_0,b_0,f_0),(t_1,n_1,b_1,f_1)));
            }
        }
    }
    for ((t_0,n_0,b_0,f_0),(t_1,n_1,b_1,f_1)) in coords {
        rectangle4(&mut data, t_0,n_0, t_1,n_1,b_0,f_0,b_1,f_1,width);
    }
    Ok((data,depths))
}

impl DynamicShape for Rectangles {
    fn any_dynamic(&self) -> bool {
        lock!(self.data).any_dynamic()
    }

    fn recompute(&mut self, gl: &WebGlGlobal) -> Result<(),Error> {
        lock!(self.data).recompute(gl)
    }
}
