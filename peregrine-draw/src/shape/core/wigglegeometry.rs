use peregrine_data::Allotment;

use super::super::layers::layer::{ Layer };
use super::super::layers::patina::PatinaProcessName;
use crate::shape::layers::geometry::{GeometryAdder, GeometryYielder};
use crate::shape::layers::patina::PatinaYielder;
use crate::webgl::{AttribHandle, ProcessBuilder, ProcessStanzaAddable, ProcessStanzaArray, ProgramBuilder};
use super::super::util::arrayutil::{ interleave_pair, apply_left };
use crate::util::message::Message;

const THICKNESS: f64 = 1.; // XXX

pub(crate) fn make_wiggle(layer: &mut Layer, geometry_yielder: &mut GeometryYielder, patina_yielder: &mut dyn PatinaYielder,
                    start: f64, end: f64, yy: &[Option<f64>], height: f64,
                    allotment: &Allotment, left: f64, depth: i8)-> Result<(ProcessStanzaArray,usize),Message> {
    let process = layer.get_process_builder(geometry_yielder,patina_yielder)?;
    let yy = yy.iter().map(|y| y.map(|y| ((1.-y)*height))).collect::<Vec<_>>();
    let yy = allotment.transform_yy(&yy);
    let adder = geometry_yielder.get_adder::<GeometryAdder>()?;
    match adder {
        GeometryAdder::Wiggle(w) => {
            w.add_wiggle(process,start,end,&yy,height,left,depth)
        },
        _ => { return Err(Message::CodeInvariantFailed(format!("bad adder"))) }
    }
}

#[derive(Clone)]
pub struct WiggleAdder {
    data: AttribHandle,
    depth: AttribHandle
}

impl WiggleAdder {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<WiggleAdder,Message> {
        Ok(WiggleAdder {
            data: builder.get_attrib_handle("aData")?,
            depth: builder.get_attrib_handle("aDepth")?
        })
    }

    pub(crate) fn add_wiggle(&self, process: &mut ProcessBuilder, start: f64, end: f64, yy: &[Option<f64>], height: f64, left: f64, depth: i8) -> Result<(ProcessStanzaArray,usize),Message> {
        if yy.len() > 1 {
            let mut pusher = WigglePusher {
                prev_active: true,
                x_step: (end-start+1.)/(yy.len() as f64),
                x_pos: start,
                y_height: height,
                x: vec![],
                y: vec![]
            };
            for y_pos in yy.iter() {
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
            let gl_depth = 1.0 - (depth as f32+128.) / 255.;
            array.add_n(&self.depth,vec![gl_depth],1)?;
            Ok((array,pusher.x.len()))
        } else {
            Ok((process.get_stanza_builder().make_array(0)?,0))
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
        if let (Some(last_x),Some(last_y)) = (self.x.last().cloned(),self.y.last().cloned()) {
            self.x.push(last_x);
            self.y.push(last_y);    
        }
    }

    fn active(&mut self, y: f64) {
        if !self.prev_active {
            self.cap();
        }
        self.x.push(self.x_pos);
        self.y.push(y-THICKNESS);
        if !self.prev_active {
            self.cap();
        }
        self.x.push(self.x_pos);
        self.y.push(y+THICKNESS);
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
    variety: WiggleAdder,
    patina: PatinaProcessName
}

impl WiggleGeometry {
    pub(crate) fn new(patina: &PatinaProcessName, variety: &WiggleAdder) -> Result<WiggleGeometry,Message> {
        Ok(WiggleGeometry { variety: variety.clone(), patina: patina.clone() })
    }
}
