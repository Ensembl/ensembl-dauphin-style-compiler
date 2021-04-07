use web_sys::WebGlRenderingContext;

use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaAddable, Program, ProcessStanzaArray, GPUSpec };
use super::super::util::arrayutil::{ interleave_pair, apply_left };
use crate::util::message::Message;

const THICKNESS: f64 = 1.; // XXX

#[derive(Clone)]
pub struct WiggleProgram {
    data: AttribHandle
}

impl WiggleProgram {
    pub(crate) fn new(program: &Program) -> Result<WiggleProgram,Message> {
        Ok(WiggleProgram {
            data: program.get_attrib_handle("aData")?,
        })
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

    pub(crate) fn add_wiggle(&self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, layer: &mut Layer, start: f64, end: f64, yy: Vec<Option<f64>>, height: f64) -> Result<ProcessStanzaArray,Message> {
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
            let mut array = layer.make_array(context,gpuspec,&GeometryProcessName::Wiggle,&self.patina,pusher.x.len())?;
            apply_left(&mut pusher.x,layer.left());
            array.add(&self.variety.data,interleave_pair(&pusher.x,&pusher.y),2)?;
            Ok(array)
        } else {
            Ok(layer.make_array(context,gpuspec,&GeometryProcessName::Wiggle,&self.patina,0)?)
        }
    }
}
