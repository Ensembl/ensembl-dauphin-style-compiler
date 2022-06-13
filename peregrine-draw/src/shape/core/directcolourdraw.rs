use crate::{shape::{layers::patina::{PatinaProcess, PatinaProcessName, PatinaAdder, PatinaYielder}, util::iterators::eoe_throw}, webgl::{ AttribHandle, ProcessStanzaAddable, ProgramBuilder }};
use peregrine_data::{DirectColour};
use peregrine_toolkit::eachorevery::eachorevery::EachOrEvery;
use super::super::util::arrayutil::scale_colour;
use crate::util::message::Message;

#[derive(Clone)]
pub struct DirectProgram {
    colour: AttribHandle
}

impl DirectProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<DirectProgram,Message> {
        Ok(DirectProgram {
            colour: builder.get_attrib_handle("aVertexColour")?
        })
    }
}

#[derive(Clone)]
pub struct DirectColourDraw(DirectProgram);

impl DirectColourDraw {
    pub(crate) fn new(variety: &DirectProgram) -> Result<DirectColourDraw,Message> {
        Ok(DirectColourDraw(variety.clone()))
    }

    pub(crate) fn direct(&self, addable: &mut dyn ProcessStanzaAddable, colours: &EachOrEvery<DirectColour>, vertexes: usize, count: usize) -> Result<(),Message> {
        let mut codes = vec![];
        let colours = eoe_throw("direct colours",colours.iter(count))?;
        for c in colours {
            for _ in 0..vertexes {
                codes.push(scale_colour(c.0));
                codes.push(scale_colour(c.1));
                codes.push(scale_colour(c.2));
                codes.push(scale_colour(c.3));
            }
        }
        addable.add_n(&self.0.colour,codes,4)?;
        Ok(())
    }
}

pub(crate) struct DirectYielder {
    patina_process_name: PatinaProcessName,
    draw: Option<DirectColourDraw>
}

impl DirectYielder {
    pub(crate) fn new() -> DirectYielder {
        DirectYielder { 
            patina_process_name: PatinaProcessName::Direct,
            draw: None
        }
    }

    pub(crate) fn draw(&self) -> Result<&DirectColourDraw,Message> {
        self.draw.as_ref().ok_or_else(|| Message::CodeInvariantFailed(format!("using accessor without setting")))
    }
}

impl PatinaYielder for DirectYielder {
    fn name(&self) -> &PatinaProcessName { &self.patina_process_name }

    fn make(&mut self, builder: &ProgramBuilder) -> Result<PatinaAdder,Message> {
        Ok(PatinaAdder::Direct(DirectProgram::new(builder)?))
    }
    
    fn set(&mut self, program: &PatinaProcess) -> Result<(),Message> {
        self.draw = Some(match program {
            PatinaProcess::Direct(d) => d,
            _ => { Err(Message::CodeInvariantFailed(format!("mismatched program: colour")))? }
        }.clone());
        Ok(())
    }
}
