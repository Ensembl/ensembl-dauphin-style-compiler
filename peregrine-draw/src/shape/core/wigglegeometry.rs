use web_sys::WebGlRenderingContext;

use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProgramName;
use super::super::layers::patina::PatinaProcessName;
use crate::shape::layers::geometry::{GeometryProgram, GeometryYielder};
use crate::webgl::{ AttribHandle, ProcessBuilder, ProcessStanzaAddable, Program, ProcessStanzaArray, GPUSpec, ProgramBuilder };
use super::super::util::arrayutil::{ interleave_pair, apply_left };
use crate::util::message::Message;

const THICKNESS: f64 = 1.; // XXX

#[derive(Clone)]
pub struct WiggleProgram {
    data: AttribHandle
}

impl WiggleProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<WiggleProgram,Message> {
        Ok(WiggleProgram {
            data: builder.get_attrib_handle("aData")?,
        })
    }

    pub(crate) fn add_wiggle(&self, process: &mut ProcessBuilder, start: f64, end: f64, yy: Vec<Option<f64>>, height: f64, left: f64) -> Result<ProcessStanzaArray,Message> {
        if yy.len() > 1 {
            let mut pusher = WigglePusher {
                prev_active: true,
                x_step: (end-start)/(yy.len() as f64),
                x_pos: start,
                y_height: height,
                x: vec![],
                y: vec![]
            };
            for y_pos in &yy {
                if let Some(y_pos) = y_pos {
                    pusher.active(*y_pos);
                } else {
                    pusher.inactive();
                }
            }
            let mut array = process.get_stanza_builder().make_array(pusher.x.len())?;
            apply_left(&mut pusher.x,left);
            // XXX apply left earlier to avoid clone
            let x = pusher.x.iter().map(|x| *x as f32).collect::<Vec<_>>();
            let y = pusher.x.iter().map(|x| *x as f32).collect::<Vec<_>>();
            array.add(&self.data,interleave_pair(&x,&y),2)?;
            Ok(array)
        } else {
            Ok(process.get_stanza_builder().make_array(0)?)
        }
    }
}

struct WigglePusher {
    prev_active: bool,
    x_step: f64,
    y_height: f64,
    x_pos: f64,
    x: Vec<f64>,
    y: Vec<f64>
}

impl WigglePusher {
    fn cap(&mut self) {
        self.x.push(*self.x.last().unwrap());
        self.y.push(*self.y.last().unwrap());
    }

    fn active(&mut self, y: f64) {
        if !self.prev_active {
            self.cap();
        }
        self.x.push(self.x_pos);
        self.y.push(y*self.y_height-THICKNESS);
        if !self.prev_active {
            self.cap();
        }
        self.x.push(self.x_pos);
        self.y.push(y*self.y_height+THICKNESS);
        self.x_pos += self.x_step;
        self.prev_active = true;
    }

    fn inactive(&mut self) {
        self.x_pos += self.x_step;
        self.prev_active = false;        
    }
}

#[derive(Clone)]
pub struct WiggleGeometry {
    variety: WiggleProgram,
    patina: PatinaProcessName
}

impl WiggleGeometry {
    pub(crate) fn new(patina: &PatinaProcessName, variety: &WiggleProgram) -> Result<WiggleGeometry,Message> {
        Ok(WiggleGeometry { variety: variety.clone(), patina: patina.clone() })
    }
}

struct WiggleAccessor {
    geometry_program_name: GeometryProgramName,
    wiggles: Option<WiggleProgram>
}

impl<'a> GeometryYielder for WiggleAccessor {
    fn name(&self) -> &GeometryProgramName { &self.geometry_program_name }

    fn make(&mut self, builder: &ProgramBuilder) -> Result<GeometryProgram,Message> {
        self.geometry_program_name.make_geometry_program(builder)
    }

    fn set(&mut self, program: &GeometryProgram) -> Result<(),Message> {
        self.wiggles = Some(match program {
            GeometryProgram::Wiggle(w) => w,
            _ => { Err(Message::CodeInvariantFailed(format!("mismatched program: wiggle")))? }
        }.clone());
        Ok(())
    }
}

impl WiggleAccessor {
    pub(crate) fn wiggles(&self) -> Result<&WiggleProgram,Message> {
        self.wiggles.as_ref().ok_or_else(|| Message::CodeInvariantFailed(format!("using accessor without setting")))
    }
}