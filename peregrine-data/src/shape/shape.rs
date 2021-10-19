use std::hash::Hash;
use super::core::{ Patina, Pen, Plotter };
use std::cmp::{ max, min };
use crate::Assets;
use crate::Flattenable;
use crate::HoleySpaceBase;
use crate::HoleySpaceBaseArea;
use crate::allotment::allotment::CoordinateSystem;
use crate::allotment::allotmentrequest::AllotmentRequest;
use crate::util::ringarray::DataFilter;

#[derive(Clone,Hash,PartialEq,Eq)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum FilterMinMax {
    Base,
    None
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Shape {
    Text(HoleySpaceBase,Pen,Vec<String>,Vec<AllotmentRequest>,CoordinateSystem),
    Image(HoleySpaceBase,Vec<String>,Vec<AllotmentRequest>,CoordinateSystem),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,AllotmentRequest,CoordinateSystem),
    SpaceBaseRect(HoleySpaceBaseArea,Patina,Vec<AllotmentRequest>,CoordinateSystem)
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
    pub fn register_space(&self, assets: &Assets) {
        match self {
            Shape::SpaceBaseRect(area,_,allotments,_) => {
                let (area,_) = area.extract();
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    allotment.register_usage(top_left.normal.ceil() as i64);
                    allotment.register_usage(bottom_right.normal.ceil() as i64);
                }
            },
            Shape::Text(position,pen,_,allotments,_) => {
                let (position,_) = position.extract();
                for (position,allotment) in position.iter().zip(allotments.iter()) {
                    allotment.register_usage((*position.normal + pen.size() as f64).ceil() as i64);
                }
            },
            Shape::Image(position,asset,allotments,_) => {
                let (position,_) = position.extract();
                for (position,(allotment,asset_name)) in position.iter().zip(allotments.iter().cycle().zip(asset.iter().cycle())) {
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
            Shape::SpaceBaseRect(area,patina,allotments,filter_min_max) => {
                let filter = area.make_base_filter(min_value,max_value);
                Shape::SpaceBaseRect(area.filter(&filter),patina.filter(&filter),filter.filter(allotments),filter_min_max.clone())
            },
            Shape::Text(position,pen,text,allotments,filter_min_max) => {
                let filter = position.make_base_filter(min_value,max_value);
                Shape::Text(position.filter(&filter),pen.filter(&filter),filter.filter(text),filter.filter(allotments),filter_min_max.clone())
            },
            Shape::Image(position,asset,allotments,filter_min_max) => {
                let filter = position.make_base_filter(min_value,max_value);
                Shape::Image(position.filter(&filter),filter.filter(asset),filter.filter(allotments),filter_min_max.clone())
            },
            Shape::Wiggle((x_start,x_end),y,plotter,allotment,filter_min_max) => {
                let (aim_min,aim_max,new_y) = wiggle_filter(min_value,max_value,*x_start,*x_end,y);
                Shape::Wiggle((aim_min,aim_max),new_y,plotter.clone(),allotment.clone(),filter_min_max.clone())
            }
        }
    }

    pub fn demerge_by_allotment<X: Hash + PartialEq + Eq,T>(self, cb: T) -> Vec<(X,Shape)> where T: Fn(&AllotmentRequest) -> X {
        let mut out = vec![];
        match self {
            Shape::Wiggle(range,y,plotter,allotment,filter_min_max) => {
                let group = cb(&allotment);
                out.push((group,Shape::Wiggle(range,y,plotter.clone(),allotment.clone(),filter_min_max)));
            },
            Shape::Text(spacebase,pen,texts,allotment,filter_min_max) => {
                let demerge = DataFilter::demerge(&allotment, cb);
                for (draw_group,mut filter) in demerge {
                    filter.set_size(spacebase.len());
                    out.push((draw_group,Shape::Text(spacebase.filter(&filter),pen.filter(&filter),filter.filter(&texts),filter.filter(&allotment),filter_min_max.clone())));
                }
            },
            Shape::Image(spacebase,images,allotment,filter_min_max) => {
                let demerge = DataFilter::demerge(&allotment, cb);
                for (draw_group,mut filter) in demerge {
                    filter.set_size(spacebase.len());
                    out.push((draw_group,Shape::Image(spacebase.filter(&filter),filter.filter(&images),filter.filter(&allotment),filter_min_max.clone())));
                }
            },
            Shape::SpaceBaseRect(area,patina,allotment,filter_min_max) => {
                let demerge = DataFilter::demerge(&allotment, cb);
                for (draw_group,mut filter) in demerge {
                    filter.set_size(area.len());
                    out.push((draw_group,Shape::SpaceBaseRect(area.filter(&filter),patina.clone(),filter.filter(&allotment),filter_min_max.clone())));
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
            Shape::SpaceBaseRect(area,patina,allotments,filter_min_max) => {
                let mut allotment_iter = allotments.iter();
                let mut filter = DataFilter::new(&mut allotment_iter, |a| !a.is_dustbin());
                filter.set_size(area.len());
                Shape::SpaceBaseRect(area.filter(&filter),patina.filter(&filter),filter.filter(&allotments),filter_min_max)
            },
            Shape::Text(position,pen,text,allotments,filter_min_max) => {
                let mut allotment_iter = allotments.iter();
                let mut filter = DataFilter::new(&mut allotment_iter, |a| !a.is_dustbin());
                filter.set_size(position.len());
                Shape::Text(position.filter(&filter),pen.filter(&filter),filter.filter(&text),filter.filter(&allotments),filter_min_max)
            },
            Shape::Image(position,asset,allotments,filter_min_max) => {
                let mut allotment_iter = allotments.iter();
                let mut filter = DataFilter::new(&mut allotment_iter, |a| !a.is_dustbin());
                filter.set_size(position.len());
                Shape::Image(position.filter(&filter),filter.filter(&asset),filter.filter(&allotments),filter_min_max)
            },
            Shape::Wiggle(x,mut y,plotter,allotment,filter_min_max) => {
                if allotment.is_dustbin() { y = vec![]; }
                Shape::Wiggle(x,y,plotter.clone(),allotment.clone(),filter_min_max)
            }
        }
    }
}
