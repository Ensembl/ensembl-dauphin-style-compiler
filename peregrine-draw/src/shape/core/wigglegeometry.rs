use peregrine_toolkit::error::Error;
use crate::{webgl::{AttribHandle, ProcessBuilder, ProcessStanzaAddable, ProcessStanzaArray }, shape::layers::geometry::{GeometryFactory, GeometryProcessName}};
use super::super::util::arrayutil::{ interleave_pair, apply_left };

const THICKNESS: f64 = 1.; // TODO configurable

#[derive(Clone)]
pub struct WiggleAdder {
    data: AttribHandle,
    depth: AttribHandle
}

impl WiggleAdder {
    fn new(process: &ProcessBuilder) -> Result<WiggleAdder,Error> {
        let program = process.program_builder();
        Ok(WiggleAdder {
            data: program.get_attrib_handle("aData")?,
            depth: program.get_attrib_handle("aDepth")?
        })
    }

    pub(crate) fn add_wiggle(&self, process: &mut ProcessBuilder, start: f64, end: f64, yy: &[Option<f64>], left: f64, depth: i8) -> Result<(ProcessStanzaArray,usize),Error> {
        if yy.len() > 1 {
            let mut pusher = WigglePusher {
                prev_active: true,
                x_step: (end-start+1.)/(yy.len() as f64),
                x_pos: start,
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

pub(crate) struct WiggleAdderFactory;

impl WiggleAdderFactory {
    pub(crate) fn new() -> WiggleAdderFactory { WiggleAdderFactory }

    pub(crate) fn make(&self, process: &mut ProcessBuilder) -> Result<WiggleAdder,Error> {
        WiggleAdder::new(process)
    }
}

impl GeometryFactory for WiggleAdderFactory {
    fn geometry_name(&self) -> GeometryProcessName {
        GeometryProcessName::Wiggle
    }
}

struct WigglePusher {
    prev_active: bool,
    x_step: f64,
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
