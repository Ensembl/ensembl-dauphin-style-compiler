use std::{sync::{Arc, Mutex}, f64::consts::PI};
use eachorevery::EachOrEvery;
use peregrine_data::{SpaceBase, AuxLeaf, reactive::{Observer, Observable}, SpaceBasePoint};
use peregrine_toolkit::{ error::{Error, err_web_drop}, lock};
use crate::{webgl::{ProcessStanzaElements, ProcessBuilder, global::WebGlGlobal}, shape::{layers::{drawing::DynamicShape, geometry::{GeometryProcessName, GeometryFactory}}}};
use super::{triangleadder::TriangleAdder, drawgroup::DrawGroup, rectangles::fix_normal_unpacked};

/* Vertex 0 sits in the middle, the others round the edge. We draw triangles
 * (0,1,2), (0,2,3), ... (0,n-1,n), (0,n,1)
 */

pub(super) fn apply_wobble<A: Clone>(pos: &SpaceBase<f64,A>, wobble: &SpaceBase<Observable<'static,f64>,()>) -> SpaceBase<f64,A> {
    let wobble = wobble.map_all(|obs| obs.get());
    pos.merge(wobble,SpaceBasePoint {
        base: &|a,b| { *a+*b },
        normal: &|a,b| { *a+*b },
        tangent: &|a,b| { *a+*b },
        allotment: &|a,_| { a.clone() }
    })
}

fn apply_any_wobble<A: Clone>(spacebase: &SpaceBase<f64,A>, wobble: &Option<SpaceBase<Observable<'static,f64>,()>>) -> SpaceBase<f64,A> {
    if let Some(wobble) = wobble {
        apply_wobble(spacebase,&wobble)
    } else {
        spacebase.clone()
    }
}

fn calc_points(points: usize, angle: f32) -> Vec<(f64,f64)> {
    let mut out = vec![];
    let mut theta = (angle as f64) * PI / 180.;
    let delta_theta = 2. * PI / (points as f64);
    for _ in 0..points {
        let (y,x) = theta.sin_cos();
        out.push((y,x));
        theta += delta_theta;
    }
    out
}

pub(crate) struct SolidPolygonData {
    elements: ProcessStanzaElements,
    wobble: Option<SpaceBase<Observable<'static,f64>,()>>,
    wobbled: Arc<Mutex<SpaceBase<f64,AuxLeaf>>>,
    centre: SpaceBase<f64,AuxLeaf>,
    points: Vec<(f64,f64)>,
    depth: i8,
    radius: EachOrEvery<f64>,
    adder: TriangleAdder,
    left: f64,
    group: DrawGroup
}

/* eg a pentagon is (0,1,2) (0,2,3) (0,2,4) */

impl SolidPolygonData {
    fn new(builder: &mut ProcessBuilder, 
                centre: &SpaceBase<f64,AuxLeaf>, radius: &EachOrEvery<f64>,
                points: usize, angle: f32, depth: i8,
                left: f64, group: &DrawGroup,
                wobble: Option<SpaceBase<Observable<'static,f64>,()>>
            )-> Result<SolidPolygonData,Error> {
        let adder = TriangleAdder::new(builder)?;
        let mut indexes = vec![];
        for i in 0..(points as u16)-2 {
            indexes.push(0);
            indexes.push(i+1);
            indexes.push(i+2);
        }
        let elements = builder.get_stanza_builder().make_elements(centre.len(),&indexes)?;
        Ok(SolidPolygonData {
            elements,
            wobble,
            centre: centre.clone(),
            radius: radius.clone(),
            points: calc_points(points,angle),
            wobbled: Arc::new(Mutex::new(centre.clone())),
            group: group.clone(),
            adder, left, depth
        })
    }

    fn wobble(&mut self) -> Option<Box<dyn FnMut() + 'static>> {
        self.wobble.as_ref().map(|wobble| {
            *lock!(self.wobbled) = apply_any_wobble(&self.centre,&Some(wobble.clone()));
            let wobble = wobble.clone();
            let centre = self.centre.clone();
            let wobbled = self.wobbled.clone();
            Box::new(move || {
                let new_centre = apply_any_wobble(&centre,&Some(wobble.clone()));
                *lock!(wobbled) = new_centre;
            }) as Box<dyn FnMut() + 'static>
        })
    }

    fn watch(&self, observer: &mut Observer<'static>) {
        if let Some(wobble) = &self.wobble {
            for obs in wobble.iter() {
                observer.observe(obs.base);
                observer.observe(obs.normal);
                observer.observe(obs.tangent);
            }
        }
    }

    pub(crate) fn elements_mut(&mut self) -> &mut ProcessStanzaElements { &mut self.elements }

    fn add_points(&self, centre: &SpaceBase<f64,AuxLeaf>, points: &Vec<(f64,f64)>) -> (Vec<f32>,Vec<f32>) {
        let mut data = vec![];
        let mut depths = vec![];
        let gl_depth = 1.0 - (self.depth as f32+128.) / 255.;
        if self.group.packed_format() {
            for (centre,radius) in centre.iter().zip(self.radius.iter(self.centre.len()).unwrap()) {
                let b = (centre.base - self.left) as f32;
                let f = gl_depth;
                for (delta_t,delta_n) in points {
                    data.push((centre.tangent + delta_t*radius) as f32);
                    data.push((centre.normal + delta_n*radius) as f32);
                    data.push(b);
                    data.push(f);
                }
            }
        } else {
            for (centre,radius) in centre.iter().zip(self.radius.iter(self.centre.len()).unwrap()) {
                let b = (centre.base - self.left) as f32;
                if self.group.coord_system().flip_xy() {
                    for (delta_t,delta_n) in points {
                        let (n,f) = fix_normal_unpacked(*centre.normal,&self.group);
                        data.push((n + delta_n*radius) as f32);
                        data.push((centre.tangent + delta_t*radius) as f32);
                        data.push(f as f32);
                        data.push(b);
                    }
                } else {
                    for (delta_t,delta_n) in points {
                        let (n,f) = fix_normal_unpacked(*centre.normal,&self.group);
                        data.push((centre.tangent + delta_t*radius) as f32);
                        data.push((n + delta_n*radius) as f32);
                        data.push(b);
                        data.push(f as f32);
                    }
                }
            }
            depths = vec![gl_depth as f32;self.points.len()*self.centre.len()];
        }
        (data,depths)
    }

    fn recompute(&mut self, _gl: &WebGlGlobal) -> Result<(),Error> {
        let wobbled_centre = lock!(self.wobbled).clone();
        let (data,depths) = self.add_points(&wobbled_centre,&self.points);
        self.adder.add_data(&mut self.elements,data,depths)?;
        if self.adder.origin_coords.is_some() {
            let origins = vec![(-1.,-1.);self.points.len()];
            let (data,_) = self.add_points(&wobbled_centre,&origins);
            self.adder.add_origin_data(&mut self.elements,data)?;    
        }
        if self.adder.run_coords.is_some() {
            let origins = vec![(1.,1.);self.points.len()];
            let (data,_) = self.add_points(&wobbled_centre,&origins);
            self.adder.add_run_data(&mut self.elements,data)?;    
        }
        Ok(())
    }
}

pub(crate) struct SolidPolygon {
    data: Arc<Mutex<SolidPolygonData>>,
    wobble: Option<Observer<'static>>
}

impl SolidPolygon {
    pub(crate) fn new(data: SolidPolygonData, gl: &WebGlGlobal) -> SolidPolygon {
        let data = Arc::new(Mutex::new(data));
        let wobble_cb = lock!(data).wobble();
        let wobble = wobble_cb.map(|cb| Observer::new_boxed(cb));
        let mut out = SolidPolygon {
            data,
            wobble
        };
        if let Some(wobble) = &mut out.wobble {
            lock!(out.data).watch(wobble);
        }
        err_web_drop(out.recompute(gl));
        out
    }
}

impl DynamicShape for SolidPolygon {
    fn any_dynamic(&self) -> bool {
        lock!(self.data).wobble.is_some()
    }

    fn recompute(&mut self, gl: &WebGlGlobal) -> Result<(),Error> {
        lock!(self.data).recompute(gl)
    }
}


pub(crate) struct SolidPolygonDataFactory {
    draw_group: DrawGroup
}

impl SolidPolygonDataFactory {
    pub(crate) fn new(draw_group: &DrawGroup) -> SolidPolygonDataFactory {
        SolidPolygonDataFactory {
            draw_group: draw_group.clone()
        }
    }
    pub(crate) fn make(&self, builder: &mut ProcessBuilder, centre: &SpaceBase<f64,AuxLeaf>, radius: &EachOrEvery<f64>,
        points: usize, angle: f32, depth: i8,
        left: f64, group: &DrawGroup,
        wobble: Option<SpaceBase<Observable<'static,f64>,()>>)-> Result<SolidPolygonData,Error> {
        SolidPolygonData::new(builder,centre,radius,points,angle,depth,left,&self.draw_group,wobble)
    }
}

impl GeometryFactory for SolidPolygonDataFactory {
    fn geometry_name(&self) -> GeometryProcessName {
        GeometryProcessName::Triangles(self.draw_group.geometry())
    }
}
