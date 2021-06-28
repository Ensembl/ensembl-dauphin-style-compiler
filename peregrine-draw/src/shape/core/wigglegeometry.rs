use peregrine_data::Allotment;

use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProgramName;
use super::super::layers::patina::PatinaProcessName;
use crate::shape::layers::geometry::{GeometryProcessName, GeometryProgramLink, GeometryYielder};
use crate::shape::layers::patina::PatinaYielder;
use crate::webgl::{AttribHandle, GPUSpec, ProcessBuilder, ProcessStanzaAddable, ProcessStanzaArray, ProcessStanzaElements, Program, ProgramBuilder};
use super::super::util::arrayutil::{ interleave_pair, apply_left };
use crate::util::message::Message;

const THICKNESS: f64 = 1.; // XXX

pub(crate) struct WiggleYielder {
    geometry_process_name: GeometryProcessName,
    link: Option<WiggleProgramLink>
}

impl<'a> GeometryYielder for WiggleYielder {
    fn name(&self) -> &GeometryProcessName { &self.geometry_process_name }

    fn set(&mut self, program: &GeometryProgramLink) -> Result<(),Message> {
        self.link = Some(match program {
            GeometryProgramLink::Wiggle(prog) => prog,
            _ => { Err(Message::CodeInvariantFailed(format!("mismatched program: wiggle")))? }
        }.clone());
        Ok(())
    }
}

impl WiggleYielder {
    pub(crate) fn new() -> WiggleYielder {
        WiggleYielder {
            geometry_process_name: GeometryProcessName::new(GeometryProgramName::Wiggle),
            link: None
        }
    }

    pub(super) fn link(&self) -> Result<&WiggleProgramLink,Message> {
        self.link.as_ref().ok_or_else(|| Message::CodeInvariantFailed(format!("using accessor without setting")))
    }
}


pub(crate) fn make_wiggle(layer: &mut Layer, geometry_yielder: &mut WiggleYielder, patina_yielder: &mut dyn PatinaYielder,
                    start: f64, end: f64, yy: Vec<Option<f64>>, height: f64,
                    allotment: &Allotment, left: f64)-> Result<ProcessStanzaArray,Message> {
    let process = layer.draw(geometry_yielder,patina_yielder)?.get_process_mut();
    let delta = allotment.position().offset() as f64;
    let array = geometry_yielder.link()?.add_wiggle(process,start,end,yy,delta,height,left)?;
    Ok(array)
}

#[derive(Clone)]
pub struct WiggleProgramLink {
    data: AttribHandle
}

impl WiggleProgramLink {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<WiggleProgramLink,Message> {
        Ok(WiggleProgramLink {
            data: builder.get_attrib_handle("aData")?,
        })
    }

    pub(crate) fn add_wiggle(&self, process: &mut ProcessBuilder, start: f64, end: f64, yy: Vec<Option<f64>>, delta: f64, height: f64, left: f64) -> Result<ProcessStanzaArray,Message> {
        if yy.len() > 1 {
            let mut pusher = WigglePusher {
                prev_active: true,
                x_step: (end-start)/(yy.len() as f64),
                x_pos: start,
                y_height: height,
                y_delta: delta,
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
            let y = pusher.y.iter().map(|y| *y as f32).collect::<Vec<_>>();
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
    y_delta: f64,
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
        let y = (1.-y)*self.y_height;
        if !self.prev_active {
            self.cap();
        }
        self.x.push(self.x_pos);
        self.y.push(y-THICKNESS+self.y_delta);
        if !self.prev_active {
            self.cap();
        }
        self.x.push(self.x_pos);
        self.y.push(y+THICKNESS+self.y_delta);
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
    variety: WiggleProgramLink,
    patina: PatinaProcessName
}

impl WiggleGeometry {
    pub(crate) fn new(patina: &PatinaProcessName, variety: &WiggleProgramLink) -> Result<WiggleGeometry,Message> {
        Ok(WiggleGeometry { variety: variety.clone(), patina: patina.clone() })
    }
}

struct WiggleAccessor {
    geometry_process_name: GeometryProcessName,
    wiggles: Option<WiggleProgramLink>
}

impl<'a> GeometryYielder for WiggleAccessor {
    fn name(&self) -> &GeometryProcessName { &self.geometry_process_name }

    fn set(&mut self, program: &GeometryProgramLink) -> Result<(),Message> {
        self.wiggles = Some(match program {
            GeometryProgramLink::Wiggle(w) => w,
            _ => { Err(Message::CodeInvariantFailed(format!("mismatched program: wiggle")))? }
        }.clone());
        Ok(())
    }
}

impl WiggleAccessor {
    pub(crate) fn wiggles(&self) -> Result<&WiggleProgramLink,Message> {
        self.wiggles.as_ref().ok_or_else(|| Message::CodeInvariantFailed(format!("using accessor without setting")))
    }
}
