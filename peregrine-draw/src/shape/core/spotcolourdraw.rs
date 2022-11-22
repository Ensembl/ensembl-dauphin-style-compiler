use crate::{shape::{layers::patina::{PatinaAdder, PatinaProcess, PatinaProcessName, PatinaYielder}, util::arrayutil::scale_colour}, webgl::{ ProcessBuilder, UniformHandle, ProgramBuilder }};
use peregrine_data::DirectColour;
use peregrine_toolkit::error::Error;

#[derive(Clone)]
pub struct SpotProgram {
    uniform: UniformHandle
}

impl SpotProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<SpotProgram,Error> {
        Ok(SpotProgram {
            uniform: builder.get_uniform_handle("uColour")?
        })
    }

    fn set_spot(&self, builder: &mut ProcessBuilder, colour: &DirectColour) -> Result<(),Error> {
        builder.set_uniform(&self.uniform,vec![
            scale_colour(colour.0),
            scale_colour(colour.1),
            scale_colour(colour.2),
            scale_colour(colour.3),
        ])
    }
}

#[derive(Clone)]
pub struct SpotColourDraw {
    variety: SpotProgram
}

impl SpotColourDraw {
    pub(crate) fn new(variety: &SpotProgram) -> Result<SpotColourDraw,Error> {
        Ok(SpotColourDraw { variety: variety.clone() })
    }

    pub(crate) fn set_spot(&self, builder: &mut ProcessBuilder, colour: &DirectColour) -> Result<(),Error> {
        self.variety.set_spot(builder,colour)
    }
}

pub(crate) struct SpotColourYielder {
    patina_process_name: PatinaProcessName,
    spot: Option<SpotColourDraw>
}

impl SpotColourYielder {
    pub(crate) fn new(colour: &DirectColour) -> SpotColourYielder {
        SpotColourYielder { 
            spot: None,
            patina_process_name: PatinaProcessName::Spot(colour.clone())
        }
    }
}

impl PatinaYielder for SpotColourYielder {
    fn name(&self) -> &PatinaProcessName { &self.patina_process_name }

    fn make(&mut self, builder: &ProgramBuilder) -> Result<PatinaAdder,Error> {
        Ok(PatinaAdder::Spot(SpotProgram::new(builder)?))
    }
    
    fn set(&mut self, program: &PatinaProcess) -> Result<(),Error> {
        self.spot = Some(match program {
            PatinaProcess::Spot(t) => t,
            _ => { Err(Error::fatal("mismatched program: texture"))? }
        }.clone());
        Ok(())
    }
}
