use crate::{shape::{layers::patina::{PatinaAdder, PatinaProcess, PatinaProcessName, PatinaYielder}, util::arrayutil::scale_colour}, webgl::{ ProcessBuilder, UniformHandle, ProgramBuilder }};
use peregrine_data::DirectColour;
use crate::util::message::Message;

#[derive(Clone)]
pub struct SpotProgram {
    uniform: UniformHandle
}

impl SpotProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<SpotProgram,Message> {
        Ok(SpotProgram {
            uniform: builder.get_uniform_handle("uColour")?
        })
    }

    fn set_spot(&self, builder: &mut ProcessBuilder, colour: &DirectColour) -> Result<(),Message> {
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
    pub(crate) fn new(variety: &SpotProgram) -> Result<SpotColourDraw,Message> {
        Ok(SpotColourDraw { variety: variety.clone() })
    }

    pub(crate) fn set_spot(&self, builder: &mut ProcessBuilder, colour: &DirectColour) -> Result<(),Message> {
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

    fn make(&mut self, builder: &ProgramBuilder) -> Result<PatinaAdder,Message> {
        Ok(PatinaAdder::Spot(SpotProgram::new(builder)?))
    }
    
    fn set(&mut self, program: &PatinaProcess) -> Result<(),Message> {
        self.spot = Some(match program {
            PatinaProcess::Spot(t) => t,
            _ => { Err(Message::CodeInvariantFailed(format!("mismatched program: texture")))? }
        }.clone());
        Ok(())
    }
}
