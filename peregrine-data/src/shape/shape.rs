use std::hash::Hash;
use std::sync::Arc;
use super::core::{ Patina, Pen, Plotter };
use std::cmp::{ max, min };
use crate::Assets;
use crate::Colour;
use crate::DataFilter;
use crate::DataMessage;
use crate::DrawnType;
use crate::EachOrEvery;
use crate::Flattenable;
use crate::HoleySpaceBase;
use crate::HoleySpaceBaseArea;
use crate::SpaceBase;
use crate::SpaceBaseArea;
use crate::allotment::allotment::CoordinateSystem;
use crate::allotment::allotmentrequest::AllotmentRequest;
use crate::util::eachorevery::eoe_throw;

pub trait ShapeDemerge {
    type X: Hash + PartialEq + Eq;

    fn categorise(&self, allotment: &AllotmentRequest) -> Self::X;
    
    fn categorise_with_colour(&self, allotment: &AllotmentRequest, _variety: &DrawnType, _colour: &Colour) -> Self::X {
        self.categorise(allotment)
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum ShapeDetails {
    Text(TextShape),
    Image(ImageShape),
    Wiggle(WiggleShape),
    SpaceBaseRect(RectangleShape)
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Shape {
    details: ShapeDetails,
    coord_system: CoordinateSystem
}

impl Shape {
    pub fn details(&self) -> &ShapeDetails { &self.details }
    pub fn coord_system(&self) -> &CoordinateSystem { &self.coord_system }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct WiggleShape {
    x_limits: (f64,f64),
    values: Arc<Vec<Option<f64>>>,
    plotter: Plotter,
    allotments: EachOrEvery<AllotmentRequest> // actually always a single allotment
}

impl WiggleShape {
    pub fn new_details(x_limits: (f64,f64), values: Vec<Option<f64>>, plotter: Plotter, allotment: AllotmentRequest) -> WiggleShape {
        WiggleShape {
            x_limits,
            values: Arc::new(values),
            plotter,
            allotments: EachOrEvery::each(vec![allotment])
        }
    }

    pub fn new(x_limits: (f64,f64), values: Vec<Option<f64>>, plotter: Plotter, allotment: AllotmentRequest) -> Result<Vec<Shape>,DataMessage> {
        let mut out = vec![];
        let details = WiggleShape::new_details(x_limits,values,plotter,allotment.clone());
        for (coord_system,mut filter) in details.allotments().demerge(|x| { x.coord_system() }) {
            out.push(Shape {
                coord_system: coord_system.clone(),
                details: ShapeDetails::Wiggle(details.clone().filter(&mut filter))
            });
        }
        Ok(out)
    }

    pub fn filter(&self, filter: &mut DataFilter) -> WiggleShape {
        let y = if filter.filter(&[()]).len() > 0 {
            self.values.clone()
        } else {
            Arc::new(vec![])
        };
        WiggleShape {
            x_limits: self.x_limits,
            values: y,
            plotter: self.plotter.clone(),
            allotments: self.allotments.clone()
        }
    }

    pub fn len(&self) -> usize { 1 }
    pub fn allotments(&self) -> &EachOrEvery<AllotmentRequest> { &self.allotments }
    pub fn range(&self) -> (f64,f64) { self.x_limits }
    pub fn values(&self) -> Arc<Vec<Option<f64>>> { self.values.clone() }
    pub fn plotter(&self) -> &Plotter { &self.plotter }
    pub fn allotment(&self) -> &AllotmentRequest { self.allotments.get(0).unwrap() }

    pub fn filter_by_allotment<F>(&self, cb: F)  -> WiggleShape where F: Fn(&AllotmentRequest) -> bool {
        self.filter(&mut self.allotments.new_filter(1,cb))
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
            allotments: self.allotments.clone()
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TextShape {
    position: HoleySpaceBase,
    pen: Pen,
    text: EachOrEvery<String>,
    allotments: EachOrEvery<AllotmentRequest>
}

impl TextShape {
    pub fn new_details(position: HoleySpaceBase, pen: Pen, text: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>) -> Option<TextShape> {
        if !allotments.compatible(position.len()) || !text.compatible(position.len()) { return None; }
        Some(TextShape {
            position, pen, text, allotments
        })
    }

    pub fn new(position: HoleySpaceBase, pen: Pen, text: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>) -> Result<Vec<Shape>,DataMessage> {
        let mut out = vec![];
        let details = eoe_throw("new_text",TextShape::new_details(position,pen,text,allotments.clone()))?;
        for (coord_system,mut filter) in allotments.demerge(|x| { x.coord_system() }) {
            out.push(Shape {
                coord_system,
                details: ShapeDetails::Text(details.clone().filter(&mut filter))
            });
        }
        Ok(out)        
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn allotments(&self) -> &EachOrEvery<AllotmentRequest> { &self.allotments }
    pub fn pen(&self) -> &Pen { &self.pen }
    pub fn holey_position(&self) -> &HoleySpaceBase { &self.position }
    pub fn position(&self) -> SpaceBase<f64> { self.position.extract().0 }

    pub fn filter(&self, filter: &mut DataFilter) -> TextShape {
        filter.set_size(self.position.len());
        TextShape {
            position: self.position.filter(filter),
            pen: self.pen.filter(&filter),
            text: self.text.filter(&filter),
            allotments: self.allotments.filter(filter)
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
    allotments: EachOrEvery<AllotmentRequest>
}

impl ImageShape {
    pub fn new_details(position: HoleySpaceBase, names: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>) -> Option<ImageShape> {
        if !allotments.compatible(position.len()) || !names.compatible(position.len()) { return None; }
        Some(ImageShape {
            position, names, allotments
        })
    }

    pub fn new(position: HoleySpaceBase, names: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>) -> Result<Vec<Shape>,DataMessage> {
        let mut out = vec![];
        let details = eoe_throw("add_image",ImageShape::new_details(position,names,allotments.clone()))?;
        for (coord_system,mut filter) in allotments.demerge(|x| { x.coord_system() }) {
            out.push(Shape {
                coord_system,
                details: ShapeDetails::Image(details.filter(&mut filter))
            });
        }
        Ok(out)        
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn allotments(&self) -> &EachOrEvery<AllotmentRequest> { &self.allotments }
    pub fn names(&self) -> &EachOrEvery<String> { &self.names }
    pub fn holey_position(&self) -> &HoleySpaceBase { &self.position }
    pub fn position(&self) -> SpaceBase<f64> { self.position.extract().0 }

    pub fn filter(&self, filter: &mut DataFilter) -> ImageShape {
        filter.set_size(self.position.len());
        ImageShape {
            position: self.position.filter(filter),
            names: self.names.filter(&filter),
            allotments: self.allotments.filter(filter)
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
    allotments: EachOrEvery<AllotmentRequest>
}

impl RectangleShape {
    pub fn new_details(area: HoleySpaceBaseArea, patina: Patina, allotments: EachOrEvery<AllotmentRequest>) -> Option<RectangleShape> {
        if !allotments.compatible(area.len()) { return None; }
        Some(RectangleShape {
            area, patina, allotments
        })
    }

    pub fn new(area: HoleySpaceBaseArea, patina: Patina, allotments: EachOrEvery<AllotmentRequest>) -> Result<Vec<Shape>,DataMessage> {
        let mut out = vec![];
        let details = eoe_throw("add_rectangles",RectangleShape::new_details(area,patina,allotments.clone()))?;
        for (coord_system,mut filter) in allotments.demerge(|x| { x.coord_system() }) {
            out.push(Shape {
                coord_system,
                details: ShapeDetails::SpaceBaseRect(details.clone().filter(&mut filter))
            });
        }
        Ok(out)        
    }

    pub fn len(&self) -> usize { self.area.len() }
    pub fn allotments(&self) -> &EachOrEvery<AllotmentRequest> { &self.allotments }
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
            allotments: self.allotments.filter(filter)
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
            Patina::Drawn(drawn_type,colours) => {
                let allotments_and_colours = self.allotments.merge(&colours).unwrap();
                allotments_and_colours.demerge(|(a,c)| 
                    cat.categorise_with_colour(a,drawn_type,c)
                )
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
        match &self.details {
            ShapeDetails::SpaceBaseRect(shape) => {
                for ((top_left,bottom_right),allotment) in shape.area().iter().zip(shape.iter_allotments()) {
                    allotment.register_usage(top_left.normal.ceil() as i64);
                    allotment.register_usage(bottom_right.normal.ceil() as i64);
                }
            },
            ShapeDetails::Text(shape) => {
                for (position,allotment) in shape.position().iter().zip(shape.iter_allotments()) {
                    allotment.register_usage((*position.normal + shape.pen().size() as f64).ceil() as i64);
                }
            },
            ShapeDetails::Image(shape) => {
                for (position,(allotment,asset_name)) in shape.position().iter().zip(shape.iter_allotments().zip(shape.iter_names())) {
                    if let Some(asset) = assets.get(asset_name) {
                        if let Some(height) = asset.metadata_u32("height") {
                            allotment.register_usage((position.normal + (height as f64)).ceil() as i64);
                        }
                    }
                }
            },
            ShapeDetails::Wiggle(shape) => {
                shape.allotment().register_usage(shape.plotter().0 as i64);
            }
        }
        Ok(())
    }

    fn test_filter_base(&self) -> bool {
        self.coord_system.is_tracking() 
    }

    pub fn is_tracking(&self, min_value: f64, max_value: f64) -> Shape {
        if !self.test_filter_base() {
            return self.clone();
        }
        let details = match &self.details {
            ShapeDetails::SpaceBaseRect(shape) => {
                ShapeDetails::SpaceBaseRect(shape.filter_by_minmax(min_value,max_value))
            },
            ShapeDetails::Text(shape) => {
                ShapeDetails::Text(shape.filter_by_minmax(min_value,max_value))
            },
            ShapeDetails::Image(shape) => {
                ShapeDetails::Image(shape.filter_by_minmax(min_value,max_value))
            },
            ShapeDetails::Wiggle(shape) => {
                ShapeDetails::Wiggle(shape.filter_by_minmax(min_value,max_value))
            }
        };
        Shape { details, coord_system: self.coord_system.clone() }
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,Shape)> where D: ShapeDemerge<X=T> {
        let coord_system = self.coord_system.clone();
        match self.details {
            ShapeDetails::Wiggle(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,s)| 
                    (x, Shape { coord_system: coord_system.clone(), details: ShapeDetails::Wiggle(s) })
                ).collect()
            },
            ShapeDetails::Text(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,s)|
                    (x, Shape { coord_system: coord_system.clone(), details: ShapeDetails::Text(s) })
                ).collect()
            },
            ShapeDetails::Image(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,s)|
                    (x, Shape { coord_system: coord_system.clone(), details: ShapeDetails::Image(s) })
                ).collect()
            },
            ShapeDetails::SpaceBaseRect(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,s)|
                    (x, Shape { coord_system: coord_system.clone(), details: ShapeDetails::SpaceBaseRect(s) })
                ).collect()
            }
        }
    }
    
    pub fn len(&self) -> usize {
        match &self.details {
            ShapeDetails::SpaceBaseRect(shape) => shape.len(),
            ShapeDetails::Text(shape) => shape.len(),
            ShapeDetails::Image(shape) => shape.len(),
            ShapeDetails::Wiggle(shape) => shape.len()
        }
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }

    pub fn remove_nulls(self) -> Shape {
        let details = match self.details {
            ShapeDetails::SpaceBaseRect(shape) => {
                ShapeDetails::SpaceBaseRect(shape.filter_by_allotment(|a| !a.is_dustbin()))
            },
            ShapeDetails::Text(shape) => {
                ShapeDetails::Text(shape.filter_by_allotment(|a| !a.is_dustbin()))
            },
            ShapeDetails::Image(shape) => {
                ShapeDetails::Image(shape.filter_by_allotment(|a| !a.is_dustbin()))
            },
            ShapeDetails::Wiggle(shape) => {
                ShapeDetails::Wiggle(shape.filter_by_allotment(|a| !a.is_dustbin()))
            }
        };
        Shape { details, coord_system: self.coord_system.clone() }
    }
}
