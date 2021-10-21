use std::hash::Hash;
use super::core::{ Patina, Pen, Plotter };
use std::cmp::{ max, min };
use crate::Assets;
use crate::DataFilter;
use crate::DataMessage;
use crate::EachOrEvery;
use crate::Flattenable;
use crate::HoleySpaceBase;
use crate::HoleySpaceBaseArea;
use crate::SpaceBase;
use crate::SpaceBaseArea;
use crate::allotment::allotment::CoordinateSystem;
use crate::allotment::allotmentrequest::AllotmentRequest;
use crate::allotment::allotmentrequest::AllotmentRequestImpl;
use crate::util::eachorevery::eoe_throw;

pub trait ShapeDemerge {
    type X: Hash + PartialEq + Eq;

    fn categorise(&self, allotment: &AllotmentRequest) -> Self::X;
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Shape {
    Text(HoleySpaceBase,Pen,Vec<String>,EachOrEvery<AllotmentRequest>,CoordinateSystem),
    Image(ImageShape),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,AllotmentRequest,CoordinateSystem),
    SpaceBaseRect(RectangleShape)
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ImageShape {
    position: HoleySpaceBase,
    names: EachOrEvery<String>,
    allotments: EachOrEvery<AllotmentRequest>,
    coord_system: CoordinateSystem
}

impl ImageShape {
    pub fn new(position: HoleySpaceBase, names: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>, coord_system: CoordinateSystem) -> Option<ImageShape> {
        if !allotments.compatible(position.len()) || !names.compatible(position.len()) { return None; }
        Some(ImageShape {
            position, names, allotments, coord_system
        })
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn allotments(&self) -> &EachOrEvery<AllotmentRequest> { &self.allotments }
    pub fn names(&self) -> &EachOrEvery<String> { &self.names }
    pub fn coord_system(&self) -> &CoordinateSystem { &self.coord_system }
    pub fn holey_position(&self) -> &HoleySpaceBase { &self.position }
    pub fn position(&self) -> SpaceBase<f64> { self.position.extract().0 }

    pub fn filter(&self, filter: &mut DataFilter) -> ImageShape {
        filter.set_size(self.position.len());
        ImageShape {
            position: self.position.filter(filter),
            names: self.names.filter(&filter),
            allotments: self.allotments.filter(filter),
            coord_system: self.coord_system.clone()
        }
    }

    pub fn iter_allotments(&self) -> impl Iterator<Item=&AllotmentRequest> {
        self.allotments.iter(self.position.len()).unwrap()
    }

    pub fn iter_names(&self) -> impl Iterator<Item=&String> {
        self.names.iter(self.position.len()).unwrap()
    }

    pub fn filter_by_minmax(&self, min: f64, max: f64) -> ImageShape {
        self.filter(&mut self.position.make_base_filter(min,max))
    }

    pub fn filter_by_allotment<F>(&self, cb: F)  -> ImageShape where F: Fn(&AllotmentRequest) -> bool {
        self.filter(&mut self.allotments.new_filter(self.position.len(),cb))
    }

    pub fn demerge_by_allotment<X: Hash+PartialEq+Eq,F>(&self, cb: F) -> Vec<(X,ImageShape)> where F: Fn(&AllotmentRequest) -> X {
        let demerge = self.allotments.demerge(cb);
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            out.push((draw_group,self.filter(&mut filter)));
        }
        out
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct RectangleShape {
    area: HoleySpaceBaseArea,
    patina: Patina,
    allotments: EachOrEvery<AllotmentRequest>,
    coord_system: CoordinateSystem
}

impl RectangleShape {
    pub fn new(area: HoleySpaceBaseArea, patina: Patina, allotments: EachOrEvery<AllotmentRequest>, coord_system: CoordinateSystem) -> Option<RectangleShape> {
        if !allotments.compatible(area.len()) { return None; }
        Some(RectangleShape {
            area, patina, allotments, coord_system
        })
    }

    pub fn len(&self) -> usize { self.area.len() }
    pub fn allotments(&self) -> &EachOrEvery<AllotmentRequest> { &self.allotments }
    pub fn coord_system(&self) -> &CoordinateSystem { &self.coord_system }
    pub fn patina(&self) -> &Patina { &self.patina }
    pub fn holey_area(&self) -> &HoleySpaceBaseArea { &self.area }
    pub fn area(&self) -> SpaceBaseArea<f64> { self.area.extract().0 }

    pub fn iter_allotments(&self) -> impl Iterator<Item=&AllotmentRequest> {
        self.allotments.iter(self.area.len()).unwrap()
    }

    fn filter(&self, filter: &mut DataFilter) -> RectangleShape {
        filter.set_size(self.area.len());
        RectangleShape {
            area: self.area.filter(filter),
            patina: self.patina.filter(filter),
            allotments: self.allotments.filter(filter),
            coord_system: self.coord_system.clone()
        }
    }

    pub fn filter_by_minmax(&self, min: f64, max: f64) -> RectangleShape {
        self.filter(&mut self.area.make_base_filter(min,max))
    }

    pub fn filter_by_allotment<F>(&self, cb: F)  -> RectangleShape where F: Fn(&AllotmentRequest) -> bool {
        self.filter(&mut self.allotments.new_filter(self.area.len(),cb))
    }

    pub fn demerge_by_allotment<X: Hash+PartialEq+Eq,F>(&self, cb: F) -> Vec<(X,RectangleShape)> where F: Fn(&AllotmentRequest) -> X {
        let demerge = self.allotments.demerge(cb);
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            out.push((draw_group,self.filter(&mut filter)));
        }
        out
    }
}

const SCALE : i64 = 100; // XXX configurable

fn wiggle_filter(wanted_min: f64, wanted_max: f64, got_min: f64, got_max: f64, y: &[Option<f64>]) -> (f64,f64,Vec<Option<f64>>) {
    if y.len() == 0 { return (wanted_min,wanted_max,vec![]); }
    /* truncation */
    let aim_min = if wanted_min < got_min { got_min } else { wanted_min }; // ie invariant: aim_min >= got_min
    let aim_max = if wanted_max > got_max { got_max } else { wanted_max }; // ie invariant: aim_max <= got_max
    let pitch = (got_max-got_min)/(y.len() as f64);
    let left_truncate = ((aim_min-got_min)/pitch).floor() as i64 - 1;
    let right_truncate = ((got_max-aim_max)/pitch).floor() as i64 - 1;
    let y_len = y.len() as i64;
    let left = min(max(left_truncate,0),y_len);
    let right = max(left,min(max(0,y_len-right_truncate),y_len));
    /* weeding */
    let y = if right-left+1 > SCALE*2 {
        let mut y2 = vec![];
        let got = right - left + 1;
        for (i,v) in y[(left as usize)..(right as usize)].iter().enumerate() {
            if i as i64 * SCALE / got as i64 - y2.len() as i64 > 1 {
                y2.push(v.clone());
            }
        }
        y2
    } else {
        y[(left as usize)..(right as usize)].to_vec()
    };
    (aim_min,aim_max,y)
}

impl Shape {
    pub fn register_space(&self, assets: &Assets) -> Result<(),DataMessage> {
        match self {
            Shape::SpaceBaseRect(shape) => {
                for ((top_left,bottom_right),allotment) in shape.area().iter().zip(shape.iter_allotments()) {
                    allotment.register_usage(top_left.normal.ceil() as i64);
                    allotment.register_usage(bottom_right.normal.ceil() as i64);
                }
            },
            Shape::Text(position,pen,_,allotments,_) => {
                let (position,_) = position.extract();
                for (position,allotment) in position.iter().zip(eoe_throw("reg B",allotments.iter(position.len()))?) {
                    allotment.register_usage((*position.normal + pen.size() as f64).ceil() as i64);
                }
            },
            Shape::Image(shape) => {
                for (position,(allotment,asset_name)) in shape.position().iter().zip(shape.iter_allotments().zip(shape.iter_names())) {
                    if let Some(asset) = assets.get(asset_name) {
                        if let Some(height) = asset.metadata_u32("height") {
                            allotment.register_usage((position.normal + (height as f64)).ceil() as i64);
                        }
                    }
                }
            },
            Shape::Wiggle(_,_,plotter,allotment,_) => {
                allotment.register_usage(plotter.0 as i64);
            }
        }
        Ok(())
    }

    fn test_filter_base(&self) -> bool {
        let coord_system = match self {
            Shape::SpaceBaseRect(shape) => shape.coord_system(),
            Shape::Text(_,_,_,_,coord_system) => coord_system,
            Shape::Image(shape) => shape.coord_system(),
            Shape::Wiggle(_,_,_,_,coord_system) => coord_system
        };
        coord_system.is_tracking() 
    }

    pub fn is_tracking(&self, min_value: f64, max_value: f64) -> Shape {
        if !self.test_filter_base() {
            return self.clone();
        }
        match self {
            Shape::SpaceBaseRect(shape) => {
                Shape::SpaceBaseRect(shape.filter_by_minmax(min_value,max_value))
            },
            Shape::Text(position,pen,text,allotments,coord_system) => {
                let filter = position.make_base_filter(min_value,max_value);
                Shape::Text(position.filter(&filter),pen.filter(&filter),filter.filter(text),allotments.filter(&filter),coord_system.clone())
            },
            Shape::Image(shape) => {
                Shape::Image(shape.filter_by_minmax(min_value,max_value))
            },
            Shape::Wiggle((x_start,x_end),y,plotter,allotment,coord_system) => {
                let (aim_min,aim_max,new_y) = wiggle_filter(min_value,max_value,*x_start,*x_end,y);
                Shape::Wiggle((aim_min,aim_max),new_y,plotter.clone(),allotment.clone(),coord_system.clone())
            }
        }
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,Shape)> where D: ShapeDemerge<X=T> {
        let mut out = vec![];
        match self {
            Shape::Wiggle(range,y,plotter,allotment,coord_system) => {
                let group = cat.categorise(&allotment);
                out.push((group,Shape::Wiggle(range,y,plotter.clone(),allotment.clone(),coord_system)));
            },
            Shape::Text(spacebase,pen,texts,allotment,coord_system) => {
                let demerge = allotment.demerge(|a| cat.categorise(a));
                for (draw_group,mut filter) in demerge {
                    filter.set_size(spacebase.len());
                    out.push((draw_group,Shape::Text(spacebase.filter(&filter),pen.filter(&filter),filter.filter(&texts),allotment.filter(&filter),coord_system.clone())));
                }
            },
            Shape::Image(shape) => {
                return shape.demerge_by_allotment(|a| cat.categorise(a))
                    .drain(..).map(|(x,s)| (x,Shape::Image(s))).collect()
            },
            Shape::SpaceBaseRect(shape) => {
                return shape.demerge_by_allotment(|a| cat.categorise(a))
                    .drain(..).map(|(x,s)| (x,Shape::SpaceBaseRect(s))).collect()
            }
        }
        out
    }
    
    pub fn len(&self) -> usize {
        match self {
            Shape::SpaceBaseRect(shape) => shape.len(),
            Shape::Text(position,_,_,_,_) => position.len(),
            Shape::Image(shape) => shape.len(),
            Shape::Wiggle(_,y,_,_,_) => y.len()
        }
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }

    pub fn remove_nulls(self) -> Shape {
        match self {
            Shape::SpaceBaseRect(shape) => {
                Shape::SpaceBaseRect(shape.filter_by_allotment(|a| !a.is_dustbin()))
            },
            Shape::Text(position,pen,text,allotments,coord_system) => {
                let filter = allotments.new_filter(position.len(), |a| !a.is_dustbin());
                Shape::Text(position.filter(&filter),pen.filter(&filter),filter.filter(&text),allotments.filter(&filter),coord_system)
            },
            Shape::Image(shape) => {
                Shape::Image(shape.filter_by_allotment(|a| !a.is_dustbin()))
            },
            Shape::Wiggle(x,mut y,plotter,allotment,coord_system) => {
                if allotment.is_dustbin() { y = vec![]; }
                Shape::Wiggle(x,y,plotter.clone(),allotment.clone(),coord_system)
            }
        }
    }
}
