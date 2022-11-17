use peregrine_data::{ProgramShapesBuilder};
use crate::{Message};
use super::{ants::MarchingAnts, stain::Stain};

#[derive(Clone)]
pub(crate) enum Spectre {
    MarchingAnts(MarchingAnts),
    Stain(Stain),
    Compound(Vec<Spectre>)
}

impl Spectre {
    pub(crate) fn draw(&self, shapes: &mut ProgramShapesBuilder) -> Result<(),Message> {
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
