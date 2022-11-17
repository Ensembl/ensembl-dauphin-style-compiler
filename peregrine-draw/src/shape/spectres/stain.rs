use std::{collections::HashMap, sync::Arc};
use peregrine_data::{Colour, DirectColour, DrawnType, Patina, SpaceBase, SpaceBaseArea, PartialSpaceBase, reactive::{Observable}, ProgramShapesBuilder, LeafRequest};
use peregrine_toolkit::eachorevery::EachOrEvery;
use crate::{Message, run::{PgConfigKey, PgPeregrineConfig}, shape::{util::eoethrow::eoe_throw}};
use peregrine_data::reactive;

use super::{spectre::{AreaVariables, Spectre}, spectremanager::{SpectreConfigKey, SpectreManager, SpectreHandle}};

fn make_stain_param2(var: Option<&reactive::Variable<'static,f64>>, c: f64) -> Observable<'static,f64> {
    if let Some(var) = var { var.observable() } else { Observable::constant(c) }
}

fn make_stain_point<X: Clone,Y: Clone>(base: X, normal: X, tangent: X, allotment: &Y) -> Result<PartialSpaceBase<X,Y>,Message> {
    Ok(PartialSpaceBase::from_spacebase(
        eoe_throw("stain1",SpaceBase::new(&EachOrEvery::each(vec![base]),
        &EachOrEvery::each(vec![normal]),
        &EachOrEvery::each(vec![tangent]),
        &EachOrEvery::every(allotment.clone())))?
    ))
}

fn make_stain_rect_area(n1: f64, t1: f64, b1: f64, ar: &LeafRequest) -> Result<SpaceBaseArea<f64,LeafRequest>,Message> {
    let top_left = make_stain_point(0.,0.,0.,ar)?;
    let bottom_right = make_stain_point(b1,n1,t1,ar)?;
    Ok(SpaceBaseArea::new(top_left,bottom_right).unwrap())
}

fn make_stain_rect_wobble(n0: Option<&reactive::Variable<'static,f64>>, t0: Option<&reactive::Variable<'static,f64>>, n1: Option<&reactive::Variable<'static,f64>>, t1: Option<&reactive::Variable<'static,f64>>, b1: f64) -> Result<SpaceBaseArea<Observable<'static,f64>,()>,Message> {
    let n0 = make_stain_param2(n0,0.);
    let t0 = make_stain_param2(t0,0.);
    let n1 = make_stain_param2(n1,-1.);
    let t1 = make_stain_param2(t1,-1.);
    let top_left = make_stain_point(Observable::constant(0.),n0,t0,&())?;
    let bottom_right = make_stain_point(Observable::constant(b1),n1,t1,&())?;
    Ok(SpaceBaseArea::new(top_left,bottom_right).unwrap())
}

#[derive(Clone)]
pub(crate) struct Stain {
    area2: AreaVariables<'static>,
    invert: bool,
    colour: DirectColour
}

impl Stain {
    pub(crate) fn new(config: &PgPeregrineConfig, area2: &AreaVariables<'static>, manager: &SpectreManager, invert: bool) -> Result<(Arc<Stain>,SpectreHandle),Message> {
        let stain = Arc::new(Stain { 
            area2: area2.clone(),
            invert,
            colour: config.get_colour(&PgConfigKey::Spectre(SpectreConfigKey::StainColour))?
        });
        let handle = manager.add(&stain);
        Ok((stain,handle))
    }
}

impl Spectre for Stain {    
    fn draw(&self, shapes: &mut ProgramShapesBuilder) -> Result<(),Message> {
        let leaf = shapes.use_allotment("window/origin/stain").clone();
        let mut props = HashMap::new();
        props.insert("depth".to_string(),"101".to_string());
        props.insert("system".to_string(), "window".to_string());
        shapes.add_style("window/origin/stain",props);
        let mut rectangles = vec![];
        let pos2 = self.area2.tlbr().clone();
        if self.invert {
            /* top left of screen to bottom of screen, along lefthand edge of selection */
            rectangles.push(
                (make_stain_rect_area(-1.,0.,0.,&leaf)?,
                 make_stain_rect_wobble(None,None,None,Some(&pos2.1),0.)?));
            /* top right of screen to bottom of screen, along righthand edge of selection */
            rectangles.push(
                (make_stain_rect_area(-1.,-1.,1.,&leaf)?,
                 make_stain_rect_wobble(None,Some(&pos2.3),None,None,1.)?));
                 /* length of top of shape from top of screen to that shape */
            rectangles.push((
                make_stain_rect_area(0.,0.,0.,&leaf)?,
                make_stain_rect_wobble(None,Some(&pos2.1),Some(&pos2.0),Some(&pos2.3),0.)?));
                /* length of bottom of shape from bottom of shape to bottom of screen */
            rectangles.push((make_stain_rect_area(-1.,0.,0.,&leaf)?,
                            make_stain_rect_wobble(Some(&pos2.2),Some(&pos2.1),None,Some(&pos2.3),0.)?));
        } else {
            rectangles.push((make_stain_rect_area(0.,0.,0.,&leaf)?,
                            make_stain_rect_wobble(Some(&pos2.0),Some(&pos2.1),Some(&pos2.2),Some(&pos2.3),0.)?));
        }
        for (area,wobble) in rectangles.drain(..) {
            shapes.add_rectangle(area,Patina::Drawn(DrawnType::Fill,EachOrEvery::every(Colour::Direct(self.colour.clone()))),Some(wobble))
                .map_err(|e| Message::DataError(e))?;
        }
        Ok(())
    }
}
