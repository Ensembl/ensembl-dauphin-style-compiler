use std::hash::Hash;
use super::core::{ Patina, Pen, Plotter };
use std::cmp::{ max, min };
use crate::Assets;
use crate::DataMessage;
use crate::EachOrEvery;
use crate::Flattenable;
use crate::HoleySpaceBase;
use crate::HoleySpaceBaseArea;
use crate::allotment::allotment::CoordinateSystem;
use crate::allotment::allotmentrequest::AllotmentRequest;
use crate::util::eachorevery::eoe_throw;

pub trait ShapeDemerge {
    type X: Hash + PartialEq + Eq;

    fn categorise(&self, allotment: &AllotmentRequest) -> Self::X;
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Shape {
    Text(HoleySpaceBase,Pen,Vec<String>,EachOrEvery<AllotmentRequest>,CoordinateSystem),
    Image(HoleySpaceBase,Vec<String>,EachOrEvery<AllotmentRequest>,CoordinateSystem),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,AllotmentRequest,CoordinateSystem),
    SpaceBaseRect(HoleySpaceBaseArea,Patina,EachOrEvery<AllotmentRequest>,CoordinateSystem)
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
            Shape::SpaceBaseRect(area,_,allotments,_) => {
                let (area,_) = area.extract();
                for ((top_left,bottom_right),allotment) in area.iter().zip(eoe_throw("reg A",allotments.iter(area.len()))?) {
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
            Shape::Image(position,asset,allotments,_) => {
                let (position,_) = position.extract();
                for (position,(allotment,asset_name)) in position.iter().zip(eoe_throw("reg C",allotments.iter(position.len()))?.zip(asset.iter().cycle())) {
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
            Shape::SpaceBaseRect(_,_,_,coord_system) => coord_system,
            Shape::Text(_,_,_,_,coord_system) => coord_system,
            Shape::Image(_,_,_,coord_system) => coord_system,
            Shape::Wiggle(_,_,_,_,coord_system) => coord_system
        };
        coord_system.is_tracking() 
    }

    pub fn is_tracking(&self, min_value: f64, max_value: f64) -> Shape {
        if !self.test_filter_base() {
            return self.clone();
        }
        match self {
            Shape::SpaceBaseRect(area,patina,allotments,coord_system) => {
                let filter = area.make_base_filter(min_value,max_value);
                Shape::SpaceBaseRect(area.filter(&filter),patina.filter(&filter),allotments.filter(&filter),coord_system.clone())
            },
            Shape::Text(position,pen,text,allotments,coord_system) => {
                let filter = position.make_base_filter(min_value,max_value);
                Shape::Text(position.filter(&filter),pen.filter(&filter),filter.filter(text),allotments.filter(&filter),coord_system.clone())
            },
            Shape::Image(position,asset,allotments,coord_system) => {
                let filter = position.make_base_filter(min_value,max_value);
                Shape::Image(position.filter(&filter),filter.filter(asset),allotments.filter(&filter),coord_system.clone())
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
            Shape::Image(spacebase,images,allotment,coord_system) => {
                let demerge = allotment.demerge(|a| cat.categorise(a));
                for (draw_group,mut filter) in demerge {
                    filter.set_size(spacebase.len());
                    out.push((draw_group,Shape::Image(spacebase.filter(&filter),filter.filter(&images),allotment.filter(&filter),coord_system.clone())));
                }
            },
            Shape::SpaceBaseRect(area,patina,allotment,coord_system) => {
                let demerge = allotment.demerge(|a| cat.categorise(a));
                for (draw_group,mut filter) in demerge {
                    filter.set_size(area.len());
                    out.push((draw_group,Shape::SpaceBaseRect(area.filter(&filter),patina.clone(),allotment.filter(&filter),coord_system.clone())));
                }
            }
        }
        out
    }
    
    pub fn len(&self) -> usize {
        match self {
            Shape::SpaceBaseRect(area,_,_,_) => area.len(),
            Shape::Text(position,_,_,_,_) => position.len(),
            Shape::Image(position,_,_,_) => position.len(),
            Shape::Wiggle(_,y,_,_,_) => y.len()
        }
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }

    pub fn remove_nulls(self) -> Shape {
        match self {
            Shape::SpaceBaseRect(area,patina,allotments,coord_system) => {
                let filter = allotments.new_filter(area.len(), |a| !a.is_dustbin());
                Shape::SpaceBaseRect(area.filter(&filter),patina.filter(&filter),allotments.filter(&filter),coord_system)
            },
            Shape::Text(position,pen,text,allotments,coord_system) => {
                let filter = allotments.new_filter(position.len(), |a| !a.is_dustbin());
                Shape::Text(position.filter(&filter),pen.filter(&filter),filter.filter(&text),allotments.filter(&filter),coord_system)
            },
            Shape::Image(position,asset,allotments,coord_system) => {
                let filter = allotments.new_filter(position.len(), |a| !a.is_dustbin());
                Shape::Image(position.filter(&filter),filter.filter(&asset),allotments.filter(&filter),coord_system)
            },
            Shape::Wiggle(x,mut y,plotter,allotment,coord_system) => {
                if allotment.is_dustbin() { y = vec![]; }
                Shape::Wiggle(x,y,plotter.clone(),allotment.clone(),coord_system)
            }
        }
    }
}
