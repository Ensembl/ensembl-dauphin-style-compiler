use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Flattenable, HoleySpaceBase, Plotter, Shape, ShapeDemerge, ShapeDetails, SpaceBase, shape::shape::ShapeCommon, util::eachorevery::eoe_throw};
use std::{cmp::{max, min}, hash::Hash, sync::Arc};

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
        let allotments = EachOrEvery::each(vec![allotment]);
        for (coord_system,mut filter) in details.allotments().demerge(|x| { x.coord_system() }) {
            filter.set_size(1);
            out.push(Shape::new(
                eoe_throw("add_wiggle",ShapeCommon::new(filter.count(),coord_system,allotments.filter(&filter)))?,
                ShapeDetails::Wiggle(details.clone().filter(&mut filter))
            ));
        }
        Ok(out)
    }

    fn filter(&self, filter: &DataFilter) -> WiggleShape {
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

    pub fn filter_by_allotment<F>(&self, common: &mut ShapeCommon, cb: F)  -> WiggleShape where F: Fn(&AllotmentRequest) -> bool {
        let mut filter = common.allotments().new_filter(1,cb);
        filter.set_size(1);
        *common = common.filter(&filter);
        self.filter(&filter)
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon, cat: &D) -> Vec<(T,ShapeCommon,WiggleShape)> where D: ShapeDemerge<X=T> {
        let demerge = self.allotments.demerge(|a| cat.categorise(a));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            filter.set_size(1);
            let common = common_in.filter(&filter);
            out.push((draw_group,common,self.filter(&mut filter)));
        }
        out
    }

    pub fn filter_by_minmax(&self, _common: &mut ShapeCommon, min: f64, max: f64) -> WiggleShape {
        let (aim_min,aim_max,new_y) = wiggle_filter(min,max,self.x_limits.0,self.x_limits.1,&self.values);
        WiggleShape {
            x_limits: (aim_min,aim_max),
            values: Arc::new(new_y),
            plotter: self.plotter.clone(),
            allotments: self.allotments.clone()
        }
    }
}
