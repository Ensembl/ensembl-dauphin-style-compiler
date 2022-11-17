use std::{collections::HashMap, sync::Arc};
use peregrine_data::{Colour, DirectColour, DrawnType, Patina, SpaceBase, SpaceBaseArea, PartialSpaceBase, reactive::{Observable}, ProgramShapesBuilder};
use peregrine_toolkit::eachorevery::EachOrEvery;
use crate::{Message, run::{PgConfigKey, PgPeregrineConfig}, shape::{util::eoethrow::eoe_throw}};
use super::{spectre::{AreaVariables, Spectre}, spectremanager::{SpectreConfigKey, SpectreManager, SpectreHandle}};

// Must NOT be made clone, due to RAII dropping
pub(crate) struct MarchingAnts {
    area2: AreaVariables<'static>,
    width: f64,
    colour: DirectColour,
    length: u32,
    prop: f64
}

impl MarchingAnts {
    pub(crate) fn new(config: &PgPeregrineConfig, area2: &AreaVariables<'static>, manager: &SpectreManager) -> Result<(Arc<MarchingAnts>,SpectreHandle),Message> {
        let ants = Arc::new(MarchingAnts {
            area2: area2.clone(),
            width: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsWidth))?,
            colour: config.get_colour(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsColour))?,
            length: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsLength))? as u32,
            prop: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsProp))?
        });
        let handle = manager.add(&ants);
        Ok((ants,handle))
    }
}

impl Spectre for MarchingAnts {
    fn draw(&self, shapes: &mut ProgramShapesBuilder) -> Result<(),Message> {
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
