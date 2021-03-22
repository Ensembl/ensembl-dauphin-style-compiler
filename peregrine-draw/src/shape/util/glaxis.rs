use peregrine_data::{ ShipEnd, ScreenEdge };
use super::quickvec::{ vec2d, vec1d_x, vec1d_y };
use super::arrayutil::{
    sea_sign, empty_is, calculate_vertex, calculate_vertex_delta, calculate_stretch_vertex, calculate_stretch_vertex_delta,
    make_rect_elements
};
use super::iterators::{ IterRepeat, IterInterleave, IterFixed };
use std::cell::RefCell;
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::{shape::core::stage::ReadStageAxis, webgl::ProcessStanzaElements};
use super::super::layers::layer::{ Layer };
use crate::util::message::Message;

fn add(v: &[f64], delta: f64) -> Vec<f64> {
    v.iter().map(|x| x+delta).collect()
}

pub(crate) struct GLAxis {
    primary: bool,
    min: Vec<f64>,
    max: Vec<f64>,
    range: RefCell<Option<(f64,f64)>>,
    signs: Vec<f64>,
    hollow: bool,
    origin: bool
}

impl GLAxis {
    pub(crate) fn new_from_single(screen: &ScreenEdge, ship: &ShipEnd, size: &Vec<f64>, primary: bool, hollow: bool) -> GLAxis {
        let min = calculate_vertex(screen, ship, size,false);
        GLAxis {
            primary,
            min: if primary { min } else { empty_is(min,0.) },
            max: empty_is(calculate_vertex(screen, ship, size,true)  ,0.),
            signs: vec![sea_sign(screen)],
            range: RefCell::new(None),
            hollow,
            origin: false
        }
    }

    pub(crate) fn new_from_single_delta(count: usize, ship: &ShipEnd, size: &Vec<f64>, primary: bool, hollow: bool) -> GLAxis {
        let min = calculate_vertex_delta(count, ship, size,false);
        GLAxis {
            primary,
            min: if primary { min } else { empty_is(min,0.) },
            max: empty_is(calculate_vertex_delta(count, ship, size,true)  ,0.),
            signs: vec![1.],
            range: RefCell::new(None),
            hollow,
            origin: false
        }
    }

    pub(crate) fn new_single_origin(coords: &[f64], delta: f64, primary: bool, hollow: bool) -> GLAxis {
        GLAxis {
            primary,
            min: if primary { add(coords,delta) } else { empty_is(add(coords,delta),0.) },
            max: empty_is(add(coords,delta),0.),
            signs: vec![1.],
            range: RefCell::new(None),
            hollow,
            origin: true
        }
    }

    pub(crate) fn new_from_double(edge_min: &ScreenEdge, ship_min: &ShipEnd, edge_max: &ScreenEdge, ship_max: &ShipEnd, primary: bool, hollow: bool) -> GLAxis {
        let min = calculate_stretch_vertex(&edge_min,&ship_min);
        GLAxis {
            primary,
            min: if primary { min } else { empty_is(min,0.) },
            max: empty_is(calculate_stretch_vertex(&edge_max,&ship_max),0.),
            signs: vec![sea_sign(edge_min),sea_sign(edge_max)],
            range: RefCell::new(None),
            hollow,
            origin: false
        }
    }

    pub(crate) fn new_from_double_delta(count: usize, ship_min: &ShipEnd, ship_max: &ShipEnd, primary: bool, hollow: bool) -> GLAxis {
        let min = calculate_stretch_vertex_delta(count,&ship_min);
        GLAxis {
            primary,
            min: if primary { min } else { empty_is(min,0.) },
            max: empty_is(calculate_stretch_vertex_delta(count,&ship_max),0.),
            signs: vec![1.],
            range: RefCell::new(None),
            hollow,
            origin: false
        }
    }

    pub(crate) fn new_double_origin(min: &[f64], max: &Vec<f64>, delta: f64, primary: bool, hollow: bool) -> GLAxis {
        GLAxis {
            primary,
            min: if primary { add(min,delta) } else { empty_is(add(min,delta),0.) },
            max: empty_is(add(max,delta),0.),
            signs: vec![1.],
            range: RefCell::new(None),
            hollow,
            origin: true
        }
    }

    pub(crate) fn calc_range(&self) -> (f64,f64) {
        (
            self.min.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            self.max.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
        )
    }

    pub(crate) fn min(&self) -> f64 { 
        self.range.borrow_mut().get_or_insert_with(|| self.calc_range()).0
    }

    pub(crate) fn max(&self) -> f64 {
        self.range.borrow_mut().get_or_insert_with(|| self.calc_range()).1
    }

    fn min_sign(&self) -> f64 { *self.signs.first().unwrap() }
    fn max_sign(&self) -> f64 { *self.signs.last().unwrap() }

    pub(crate) fn min_screen(&self, axis: &ReadStageAxis) -> Result<f64,Message> {
        let size = axis.size()?;
        Ok(if self.min_sign() < 0. { size - self.min() } else { self.min() })
    }

    pub(crate) fn max_screen(&self, axis: &ReadStageAxis) -> Result<f64,Message> {
        let size = axis.size()?;
        Ok(if self.max_sign() < 0. { size - self.max() } else { self.max() })
    }

    pub(crate) fn min_paper(&self,  axis: &ReadStageAxis) -> Result<f64,Message> {
        let bp_per_screen = axis.bp_per_screen()?;
        let bp_left = axis.position()? - bp_per_screen;
        Ok((self.min() - bp_left) / bp_per_screen)
    }

    pub(crate) fn max_paper(&self,  axis: &ReadStageAxis) -> Result<f64,Message> {
        let bp_per_screen = axis.bp_per_screen()?;
        let bp_left = axis.position()? - bp_per_screen;
        Ok((self.max() - bp_left) / bp_per_screen)
    }

    pub(crate) fn iter<'t>(&'t self) -> Box<dyn Iterator<Item=(&f64,&f64)> + 't> {
        if self.primary {
            Box::new(self.min.iter().zip(self.max.iter().cycle()))
        } else {
            Box::new(self.min.iter().cycle().zip(self.max.iter().cycle()))
        }
    }

    pub(crate) fn iter_screen<'t>(&'t self, axis: &ReadStageAxis) -> Result<Box<dyn Iterator<Item=(f64,f64)> + 't>,Message> {
        let size = axis.size()?;
        let flip_min = self.min_sign() < 0.;
        let flip_max = self.max_sign() < 0.;
        Ok(Box::new(self.iter().map(move |(min,max)|
        (
            if flip_min { size-min } else { *min },
            if flip_max { size-max } else { *max }
        ))))
    }

    pub(crate) fn iter_paper<'t>(&'t self, axis: &ReadStageAxis) -> Result<Box<dyn Iterator<Item=(f64,f64)> + 't>,Message> {
        let size = axis.size()?;
        let bp_per_screen = axis.bp_per_screen()?;
        let bp_left = axis.position()? - bp_per_screen;
        Ok(Box::new(self.iter().map(move |(min,max)|
        (
            (*min-bp_left)/bp_per_screen*size,
            (*max-bp_left)/bp_per_screen*size
        ))))
    }

    /*                     self.sign  copies    repeat   out
     * solid double X      ab         4         2        aabb
     * solid double Y      ab         4         1        abab
     * hollow double X     ab         8         4        aaaabbbb
     * hollow double Y     ab         8         2        aabbaabb
     * solid single X      a          4         2        aaaa
     * solid single Y      a          4         1        aaaa
     * hollow single X     a          8         4        aaaaaaaa
     * hollow single Y     a          8         2        aaaaaaaa
     */
    fn sign_one_shape<'t>(&'t self, slow: bool) -> impl Iterator<Item=&'t f64> {
        let copies = if self.hollow {8} else {4};
        let mut repeats = 1;
        if slow { repeats *= 2; }
        if self.hollow { repeats *=2; }
        let signs = IterRepeat::new(self.signs.iter(),repeats).cycle();
        IterFixed::new(signs,copies)
    }

    fn sign_one_shape_2d<'t>(&'t self, y: &'t GLAxis) -> impl Iterator<Item=&'t f64> {
        IterInterleave::new(vec![self.sign_one_shape(true),y.sign_one_shape(false)])
    }

    fn signs(&self, slow: bool) -> Vec<f64> {
        let len = self.min.len();
        let items : Vec<_> = self.sign_one_shape(slow).cloned().collect();
        IterFixed::new(items.iter().cycle(),len).cloned().collect()
    }

    pub(crate) fn signs_x(&self) -> Vec<f64> { self.signs(true) }
    pub(crate) fn signs_y(&self) -> Vec<f64> { self.signs(false) }

    pub(crate) fn signs_2d<'t>(&'t self, other: &'t GLAxis) -> Vec<f64> {
        let len = if other.primary { other.min.len() } else { self.min.len() };
        let items : Vec<_> = self.sign_one_shape_2d(other).cloned().collect();
        IterFixed::new(items.iter().cycle(),len).cloned().collect()
    }

    pub(crate) fn vec2d(&self, y: &GLAxis) -> Vec<f64> {
        vec2d(self,y,self.hollow,self.origin)
    }

    pub(crate) fn vec1d_x(&self) -> Vec<f64> {
        vec1d_x(self,self.hollow,self.origin)
    }

    pub(crate) fn vec1d_y(&self) -> Vec<f64> {
        vec1d_y(self,self.hollow,self.origin)
    }

    pub(crate) fn make_elements(&self, layer: &mut Layer, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<ProcessStanzaElements,Message> {
        make_rect_elements(layer,geometry,patina,self.min.len(),self.hollow)
    }

    pub(crate) fn len(&self) -> usize { self.min.len() }
}
