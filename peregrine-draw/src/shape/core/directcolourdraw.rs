use crate::{shape::{layers::{patina::{PatinaProcessName}}, util::eoethrow::{eoe_throw2}}, webgl::{ AttribHandle, ProcessStanzaAddable, ProgramBuilder }};
use peregrine_data::{DirectColour};
use peregrine_toolkit::{eachorevery::EachOrEvery, error::Error};
use super::super::util::arrayutil::scale_colour;

#[derive(Clone)]
pub struct DirectProgram {
    colour: AttribHandle
}

impl DirectProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<DirectProgram,Error> {
        Ok(DirectProgram {
            colour: builder.get_attrib_handle("aVertexColour")?,
        })
    }
}

#[derive(Clone)]
pub struct DirectColourDraw {
    program: DirectProgram
}

impl DirectColourDraw {
    pub(crate) fn new(variety: &DirectProgram) -> Result<DirectColourDraw,Error> {
        Ok(DirectColourDraw {
            program: variety.clone()
        })
    }

    pub(crate) fn direct(&self, addable: &mut dyn ProcessStanzaAddable, colours: &EachOrEvery<DirectColour>, vertexes: usize, count: usize) -> Result<(),Error> {
        let mut codes = vec![];
        let colours = eoe_throw2("direct colours",colours.iter(count))?;
        for c in colours {
            for _ in 0..vertexes {
                codes.push(scale_colour(c.0));
                codes.push(scale_colour(c.1));
                codes.push(scale_colour(c.2));
                codes.push(scale_colour(c.3));
            }
        }
        addable.add_n(&self.program.colour,codes,4)?;
        Ok(())
    }

    pub(crate) fn direct_variable(&self, addable: &mut dyn ProcessStanzaAddable, colours: &EachOrEvery<DirectColour>, count: &[usize]) -> Result<(),Error> {
        let mut codes = vec![];
        let colours = eoe_throw2("direct colours",colours.iter(count.len()))?;
        for (c,n) in colours.zip(count.iter()) {
            for _ in 0..*n {
                codes.push(scale_colour(c.0));
                codes.push(scale_colour(c.1));
                codes.push(scale_colour(c.2));
                codes.push(scale_colour(c.3));
            }
        }
        addable.add_n(&self.program.colour,codes,4)?;
        Ok(())
    }
}

