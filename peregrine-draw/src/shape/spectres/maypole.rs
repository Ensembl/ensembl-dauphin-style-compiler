use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_data::{Colour, DirectColour, DrawnType, Patina, SpaceBase, SpaceBaseArea, PartialSpaceBase, reactive::{Observable}, ProgramShapesBuilder, LeafRequest};
use peregrine_toolkit::{eachorevery::EachOrEvery, lock};
use crate::{Message, run::{PgConfigKey, PgPeregrineConfig}, shape::{util::eoethrow::eoe_throw}};
use super::{spectre::{AreaVariables, Spectre}, spectremanager::{SpectreConfigKey, SpectreManager}};

pub(crate) struct Maypole {
    area: Mutex<AreaVariables<'static>>,
    width: f64,
    colour: DirectColour,
    length: u32,
    prop: f64
}

impl Maypole {
    pub(crate) fn new(config: &PgPeregrineConfig, manager: &SpectreManager) -> Result<Arc<Maypole>,Message> {
        let maypole = Arc::new(Maypole {
            area: Mutex::new(AreaVariables::new(manager.reactive())),
            width: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsWidth))?,
            colour: config.get_colour(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsColour))?,
            length: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsLength))? as u32,
            prop: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsProp))?
        });
        manager.add(&maypole);
        Ok(maypole)
    }

    pub(crate) fn set_position(&self, pos: f64) {
        lock!(self.area).update((0.,pos,0.,0.));
    }

    fn pole(&self, shapes: &mut ProgramShapesBuilder, leaf: &LeafRequest) -> Result<(),Message> {
        let pos2 = lock!(self.area).tlbr().clone();
        let top_left = PartialSpaceBase::from_spacebase(SpaceBase::new(
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![leaf.clone()])
        ).unwrap());
        let bottom_right =  PartialSpaceBase::from_spacebase(SpaceBase::new(
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![-1.]),
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![leaf.clone()])
        ).unwrap());
        let area = eoe_throw("w1",SpaceBaseArea::new(top_left,bottom_right))?;
        let top_left_obs = PartialSpaceBase::new(
            &EachOrEvery::each(vec![Observable::constant(0.)]),
            &EachOrEvery::each(vec![Observable::constant(0.)]),
            &EachOrEvery::every(pos2.1.observable()),
            &EachOrEvery::every(())
        );
        let bottom_right_obs = PartialSpaceBase::new(
            &EachOrEvery::each(vec![Observable::constant(0.)]),
            &EachOrEvery::each(vec![Observable::constant(0.)]),
            &EachOrEvery::every(pos2.1.observable()),
            &EachOrEvery::every(())
        );
        let obs = SpaceBaseArea::new(top_left_obs,bottom_right_obs).unwrap();
        shapes.add_rectangle(area,Patina::Drawn(
            DrawnType::Stroke(self.width),
            EachOrEvery::every(Colour::Bar(DirectColour(255,255,255,0),self.colour.clone(),(self.length,self.length),self.prop))
        ),Some(obs)).map_err(|x| Message::DataError(x))?;
        Ok(())
    }

    fn bobble(&self, shapes: &mut ProgramShapesBuilder, leaf: &LeafRequest) -> Result<(),Message> {
        let pos2 = lock!(self.area).tlbr().clone();
        let centre =  SpaceBase::new(
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![8.]),
            &EachOrEvery::each(vec![0.]),
            &EachOrEvery::each(vec![leaf.clone()])
        ).unwrap();
        let obs = SpaceBase::new(
            &EachOrEvery::each(vec![Observable::constant(0.)]),
            &EachOrEvery::each(vec![Observable::constant(0.)]),
            &EachOrEvery::every(pos2.1.observable()),
            &EachOrEvery::every(())
        ).unwrap();
        let patina = Patina::Drawn(
            DrawnType::Fill,
            EachOrEvery::every(Colour::Direct(DirectColour(255,0,0,255))));
        shapes.add_polygon(centre,EachOrEvery::every(7.),11,0.,patina,Some(obs)).map_err(|x| Message::DataError(x))?;
        Ok(())
    }
}

impl Spectre for Maypole {
    fn draw(&self, shapes: &mut ProgramShapesBuilder) -> Result<(),Message> {
        let leaf = shapes.use_allotment("window/origin/maypole").clone();
        let mut props = HashMap::new();
        props.insert("depth".to_string(),"126".to_string());
        props.insert("system".to_string(), "window".to_string());
        shapes.add_style("window/origin/maypole",props);
        self.pole(shapes,&leaf)?;
        self.bobble(shapes,&leaf)?;
        Ok(())
    }
}
