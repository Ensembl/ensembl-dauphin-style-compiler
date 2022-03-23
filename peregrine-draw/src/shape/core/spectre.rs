use std::{sync::Arc, collections::HashMap};

use peregrine_data::{Colour, DirectColour, DrawnType, Patina, SpaceBase, SpaceBaseArea, PartialSpaceBase, reactive::{Reactive, Observable}, EachOrEvery, CarriageShapeListBuilder, LeafRequest, LeafCommonStyle, Pen};
use crate::{Message, run::{PgConfigKey, PgPeregrineConfig}, shape::util::iterators::eoe_throw};
use peregrine_data::reactive;
use super::spectremanager::SpectreConfigKey;

#[derive(Clone)]
pub(crate) struct AreaVariables2<'a> {
    tlbr: (reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>),
    reactive: Reactive<'a>
}

impl<'a> AreaVariables2<'a> {
    pub(crate) fn new(reactive: &Reactive<'a>) -> AreaVariables2<'a> {
        AreaVariables2 {
            tlbr: (reactive.variable(0.),reactive.variable(0.),reactive.variable(0.),reactive.variable(0.)),
            reactive: reactive.clone()
        }
    }

    pub(crate) fn update(&mut self, tlbr: (f64,f64,f64,f64)) {
        self.tlbr.0.set(tlbr.0);
        self.tlbr.1.set(tlbr.1);
        self.tlbr.2.set(tlbr.2);
        self.tlbr.3.set(tlbr.3);
    }

    pub(crate) fn tlbr(&self) -> &(reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>) { &self.tlbr }
}

#[derive(Clone)]
pub(crate) struct MarchingAnts {
    area2: AreaVariables2<'static>,
    width: f64,
    colour: DirectColour,
    length: u32,
    prop: f64
}

impl MarchingAnts {
    pub(super) fn new(config: &PgPeregrineConfig, area2: &AreaVariables2<'static>) -> Result<MarchingAnts,Message> {
        Ok(MarchingAnts {
            area2: area2.clone(),
            width: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsWidth))?,
            colour: config.get_colour(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsColour))?,
            length: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsLength))? as u32,
            prop: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsProp))?
        })
    }

    pub(crate) fn draw(&self, shapes: &mut CarriageShapeListBuilder) -> Result<(),Message> {
        let leaf = shapes.use_allotment("window/origin/ants").clone();
        let mut props = HashMap::new();
        props.insert("depth".to_string(),"101".to_string());
        props.insert("system".to_string(), "window".to_string());
        shapes.add_style("window/origin/ants",props);
        let pos2 = self.area2.tlbr().clone();
        let top_left = PartialSpaceBase::from_spacebase(SpaceBase::new(
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![leaf.clone()])
        ).unwrap());
        let bottom_right =  PartialSpaceBase::from_spacebase(SpaceBase::new(
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![leaf.clone()])
        ).unwrap());
        let area = eoe_throw("w1",SpaceBaseArea::new(top_left,bottom_right))?;
        let top_left_obs = PartialSpaceBase::new(
            &EachOrEvery::each(vec![Observable::constant(0.)]),
            &EachOrEvery::every(pos2.0.observable()),
            &EachOrEvery::every(pos2.1.observable()),
            &EachOrEvery::every(())
        );
        let bottom_right_obs = PartialSpaceBase::new(
            &EachOrEvery::each(vec![Observable::constant(0.)]),
            &EachOrEvery::every(pos2.2.observable()),
            &EachOrEvery::every(pos2.3.observable()),
            &EachOrEvery::every(())
        );
        let obs = SpaceBaseArea::new(top_left_obs,bottom_right_obs).unwrap();
        shapes.add_rectangle(area,Patina::Drawn(
            DrawnType::Stroke(self.width),
            EachOrEvery::every(Colour::Bar(DirectColour(255,255,255,0),self.colour.clone(),(self.length,self.length),self.prop))
        ),Some(obs)).map_err(|x| Message::DataError(x))?;
        Ok(())
    }
}

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

fn make_stain_rect3(n1: f64, t1: f64, b1: f64, ar: &LeafRequest) -> Result<SpaceBaseArea<f64,LeafRequest>,Message> {
    let top_left = make_stain_point(0.,0.,0.,ar)?;
    let bottom_right = make_stain_point(b1,n1,t1,ar)?;
    Ok(SpaceBaseArea::new(top_left,bottom_right).unwrap())
}

fn make_stain_rect2(n0: Option<&reactive::Variable<'static,f64>>, t0: Option<&reactive::Variable<'static,f64>>, n1: Option<&reactive::Variable<'static,f64>>, t1: Option<&reactive::Variable<'static,f64>>, b1: f64) -> Result<SpaceBaseArea<Observable<'static,f64>,()>,Message> {
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
    area2: AreaVariables2<'static>,
    invert: bool,
    colour: DirectColour
}

impl Stain {
    pub(super) fn new(config: &PgPeregrineConfig, area2: &AreaVariables2<'static>, invert: bool) -> Result<Stain,Message> {
        Ok(Stain { 
            area2: area2.clone(),
            invert,
            colour: config.get_colour(&PgConfigKey::Spectre(SpectreConfigKey::StainColour))?
        })
    }
    
    pub(crate) fn draw(&self, shapes: &mut CarriageShapeListBuilder) -> Result<(),Message> {
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
                (make_stain_rect3(-1.,0.,0.,&leaf)?,
                 make_stain_rect2(None,None,None,Some(&pos2.1),0.)?));
            /* top right of screen to bottom of screen, along righthand edge of selection */
            rectangles.push(
                (make_stain_rect3(-1.,-1.,1.,&leaf)?,
                 make_stain_rect2(None,Some(&pos2.3),None,None,1.)?));
                 /* length of top of shape from top of screen to that shape */
            rectangles.push((
                make_stain_rect3(0.,0.,0.,&leaf)?,
                make_stain_rect2(None,Some(&pos2.1),Some(&pos2.0),Some(&pos2.3),0.)?));
                /* length of bottom of shape from bottom of shape to bottom of screen */
            rectangles.push((make_stain_rect3(-1.,0.,0.,&leaf)?,
                            make_stain_rect2(Some(&pos2.2),Some(&pos2.1),None,Some(&pos2.3),0.)?));
        } else {
            rectangles.push((make_stain_rect3(0.,0.,0.,&leaf)?,
                            make_stain_rect2(Some(&pos2.0),Some(&pos2.1),Some(&pos2.2),Some(&pos2.3),0.)?));
        }
        for (area,wobble) in rectangles.drain(..) {           
            shapes.add_rectangle(area,Patina::Drawn(DrawnType::Fill,EachOrEvery::every(Colour::Direct(self.colour.clone()))),Some(wobble))
                .map_err(|e| Message::DataError(e))?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) enum Spectre {
    MarchingAnts(MarchingAnts),
    Stain(Stain),
    Compound(Vec<Spectre>)
}

impl Spectre {
    pub(crate) fn draw(&self, shapes: &mut CarriageShapeListBuilder) -> Result<(),Message> {
        match self {
            Spectre::MarchingAnts(a) => a.draw(shapes)?,
            Spectre::Stain(a) => a.draw(shapes)?,
            Spectre::Compound(spectres) => {
                for spectre in spectres {
                    spectre.draw(shapes)?;
                }
            }
        }
        Ok(())
    }
}
