use std::hash::Hash;
use std::sync::Arc;
use super::core::{ Patina, Pen, Plotter };
use std::cmp::{ max, min };
use crate::Assets;
use crate::Colour;
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

pub trait ShapeDemerge {
    type X: Hash + PartialEq + Eq;

    fn categorise(&self, allotment: &AllotmentRequest) -> Self::X;
    
    fn categorise_with_colour(&self, allotment: &AllotmentRequest, _colour: &Colour) -> Self::X {
        self.categorise(allotment)
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Shape {
    Text(TextShape),
    Image(ImageShape),
    Wiggle(WiggleShape),
    SpaceBaseRect(RectangleShape)
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct WiggleShape {
    x_limits: (f64,f64),
    values: Arc<Vec<Option<f64>>>,
    plotter: Plotter,
    allotments: EachOrEvery<AllotmentRequest>, // actually always a single allotment
    coord_system: CoordinateSystem
}

impl WiggleShape {
    pub fn new(x_limits: (f64,f64), values: Vec<Option<f64>>, plotter: Plotter, allotment: AllotmentRequest, coord_system: CoordinateSystem) -> WiggleShape {
        WiggleShape {
            x_limits,
            values: Arc::new(values),
            plotter,
            allotments: EachOrEvery::each(vec![allotment]),
            coord_system
        }
    }

    pub fn len(&self) -> usize { self.values.len() }
    pub fn allotments(&self) -> &EachOrEvery<AllotmentRequest> { &self.allotments }
    pub fn range(&self) -> (f64,f64) { self.x_limits }
    pub fn values(&self) -> Arc<Vec<Option<f64>>> { self.values.clone() }
    pub fn coord_system(&self) -> &CoordinateSystem { &self.coord_system }
    pub fn plotter(&self) -> &Plotter { &self.plotter }
    pub fn allotment(&self) -> &AllotmentRequest { self.allotments.get(0).unwrap() }

    pub fn filter_by_allotment<F>(&self, cb: F)  -> WiggleShape where F: Fn(&AllotmentRequest) -> bool {
        let mut y = self.values.clone();
        if !cb(self.allotments.get(0).unwrap()) { y = Arc::new(vec![]); }
        WiggleShape {
            x_limits: self.x_limits,
            values: y,
            plotter: self.plotter.clone(),
            allotments: self.allotments.clone(),
            coord_system: self.coord_system.clone()
        }
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,WiggleShape)> where D: ShapeDemerge<X=T> {
        let x = cat.categorise(self.allotment());
        vec![(x,self.clone())]
    }

    pub fn filter_by_minmax(&self, min: f64, max: f64) -> WiggleShape {
        let (aim_min,aim_max,new_y) = wiggle_filter(min,max,self.x_limits.0,self.x_limits.1,&self.values);
        WiggleShape {
            x_limits: (aim_min,aim_max),
            values: Arc::new(new_y),
            plotter: self.plotter.clone(),
            allotments: self.allotments.clone(),
            coord_system: self.coord_system.clone()
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TextShape {
    position: HoleySpaceBase,
    pen: Pen,
    text: EachOrEvery<String>,
    allotments: EachOrEvery<AllotmentRequest>,
    coord_system: CoordinateSystem
}

impl TextShape {
    pub fn new(position: HoleySpaceBase, pen: Pen, text: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>, coord_system: CoordinateSystem) -> Option<TextShape> {
        if !allotments.compatible(position.len()) || !text.compatible(position.len()) { return None; }
        Some(TextShape {
            position, pen, text, allotments, coord_system
        })
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn allotments(&self) -> &EachOrEvery<AllotmentRequest> { &self.allotments }
    pub fn coord_system(&self) -> &CoordinateSystem { &self.coord_system }
    pub fn pen(&self) -> &Pen { &self.pen }
    pub fn holey_position(&self) -> &HoleySpaceBase { &self.position }
    pub fn position(&self) -> SpaceBase<f64> { self.position.extract().0 }

    pub fn filter(&self, filter: &mut DataFilter) -> TextShape {
        filter.set_size(self.position.len());
        TextShape {
            position: self.position.filter(filter),
            pen: self.pen.filter(&filter),
            text: self.text.filter(&filter),
            allotments: self.allotments.filter(filter),
            coord_system: self.coord_system.clone()
        }
    }

    pub fn iter_allotments(&self) -> impl Iterator<Item=&AllotmentRequest> {
        self.allotments.iter(self.position.len()).unwrap()
    }

    pub fn iter_texts(&self) -> impl Iterator<Item=&String> {
        self.text.iter(self.position.len()).unwrap()
    }

    pub fn filter_by_minmax(&self, min: f64, max: f64) -> TextShape {
        self.filter(&mut self.position.make_base_filter(min,max))
    }

    pub fn filter_by_allotment<F>(&self, cb: F)  -> TextShape where F: Fn(&AllotmentRequest) -> bool {
        self.filter(&mut self.allotments.new_filter(self.position.len(),cb))
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,TextShape)> where D: ShapeDemerge<X=T> {
        let demerge = self.allotments.demerge(|a| cat.categorise(a));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            out.push((draw_group,self.filter(&mut filter)));
        }
        out
    }
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
    pub fn coord_system(&self) -> &CoordinateSystem { &self.coord_system }
    pub fn names(&self) -> &EachOrEvery<String> { &self.names }
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

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,ImageShape)> where D: ShapeDemerge<X=T> {
        let demerge = self.allotments.demerge(|a| cat.categorise(a));
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

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,RectangleShape)> where D: ShapeDemerge<X=T> {
        let demerge = match &self.patina {
            Patina::Drawn(_,colours) => {
                let allotments_and_colours = self.allotments.merge(&colours).unwrap();
                allotments_and_colours.demerge(|(a,c)| cat.categorise_with_colour(a,c))
            },
            _ => {
                self.allotments.demerge(|a| cat.categorise(a))
            }
        };
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
            Shape::Text(shape) => {
                for (position,allotment) in shape.position().iter().zip(shape.iter_allotments()) {
                    allotment.register_usage((*position.normal + shape.pen().size() as f64).ceil() as i64);
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
            Shape::Wiggle(shape) => {
                shape.allotment().register_usage(shape.plotter().0 as i64);
            }
        }
        Ok(())
    }

    fn test_filter_base(&self) -> bool {
        let coord_system = match self {
            Shape::SpaceBaseRect(shape) => shape.coord_system(),
            Shape::Text(shape) => shape.coord_system(),
            Shape::Image(shape) => shape.coord_system(),
            Shape::Wiggle(shape) => shape.coord_system()
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
            Shape::Text(shape) => {
                Shape::Text(shape.filter_by_minmax(min_value,max_value))
            },
            Shape::Image(shape) => {
                Shape::Image(shape.filter_by_minmax(min_value,max_value))
            },
            Shape::Wiggle(shape) => {
                Shape::Wiggle(shape.filter_by_minmax(min_value,max_value))
            }
        }
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,Shape)> where D: ShapeDemerge<X=T> {
        match self {
            Shape::Wiggle(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,s)| (x,Shape::Wiggle(s))).collect()
            },
            Shape::Text(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,s)| (x,Shape::Text(s))).collect()
            },
            Shape::Image(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,s)| (x,Shape::Image(s))).collect()
            },
            Shape::SpaceBaseRect(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,s)| (x,Shape::SpaceBaseRect(s))).collect()
            }
        }
    }
    
    pub fn len(&self) -> usize {
        match self {
            Shape::SpaceBaseRect(shape) => shape.len(),
            Shape::Text(shape) => shape.len(),
            Shape::Image(shape) => shape.len(),
            Shape::Wiggle(shape) => shape.len()
        }
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }

    pub fn remove_nulls(self) -> Shape {
        match self {
            Shape::SpaceBaseRect(shape) => {
                Shape::SpaceBaseRect(shape.filter_by_allotment(|a| !a.is_dustbin()))
            },
            Shape::Text(shape) => {
                Shape::Text(shape.filter_by_allotment(|a| !a.is_dustbin()))
            },
            Shape::Image(shape) => {
                Shape::Image(shape.filter_by_allotment(|a| !a.is_dustbin()))
            },
            Shape::Wiggle(shape) => {
                Shape::Wiggle(shape.filter_by_allotment(|a| !a.is_dustbin()))
            }
        }
    }
}
