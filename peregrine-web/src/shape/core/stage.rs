use anyhow::{ anyhow as err };
use std::rc::Rc;
use std::sync::Mutex;
use crate::webgl::{ SourceInstrs, Uniform, GLArity, UniformHandle, Program, Process };
use super::super::layers::consts::{ PR_DEF, PR_LOW };

#[derive(Clone)]
pub(crate) struct ProgramStage {
    hpos: UniformHandle,
    vpos: UniformHandle,
    zoom: UniformHandle,
    size: UniformHandle,
    opacity: UniformHandle
}

impl ProgramStage {
    pub fn new(program: &Rc<Program>) -> anyhow::Result<ProgramStage> {
        Ok(ProgramStage {
            hpos: program.get_uniform_handle("uStageHpos")?,
            vpos: program.get_uniform_handle("uStageVpos")?,
            zoom: program.get_uniform_handle("uStageZoom")?,
            size: program.get_uniform_handle("uSize")?,
            opacity: program.get_uniform_handle("uOpacity")?
        })
    }

    pub fn apply(&self, stage: &Stage, left: f64, opacity: f64, process: &mut Process) -> anyhow::Result<()> {
        process.set_uniform(&self.hpos,vec![stage.x_position()?-left])?;
        process.set_uniform(&self.vpos,vec![stage.y_position()?])?;
        process.set_uniform(&self.zoom,vec![stage.zoom()?])?;
        let size = stage.size()?;
        process.set_uniform(&self.size,vec![size.0,size.1])?;
        process.set_uniform(&self.opacity,vec![opacity])?;
        Ok(())
    }
}

fn stage_ok<T: Clone>(x: &Option<T>) -> anyhow::Result<T> {
    x.as_ref().cloned().ok_or_else(|| err!("accseeor used on non-ready stage"))
}

#[derive(Clone)]
pub struct RedrawNeeded(Rc<Mutex<bool>>);

impl RedrawNeeded {
    pub fn new() -> RedrawNeeded {
        RedrawNeeded(Rc::new(Mutex::new(false)))
    }

    pub fn set(&mut self) {
        *self.0.lock().unwrap() = true;
    }

    pub fn test_and_reset(&mut self) -> bool {
        let mut r = self.0.lock().unwrap();
        let out = *r;
        *r = false;
        out
    }
}

// TODO greedy canvas size changes
struct StageData {
    x_position: Option<f64>,
    y_position: Option<f64>,
    zoom: Option<f64>,
    size: Option<(f64,f64)>,
    redraw_needed: RedrawNeeded 
}

#[derive(Clone)]
pub struct Stage(Rc<Mutex<StageData>>);

impl StageData {
    fn new() -> StageData { // XXX
        StageData {
            x_position: None,
            y_position: None,
            zoom: None,
            size: None,
            redraw_needed: RedrawNeeded::new()
        }
    }

    fn changed(&mut self) {
        let ready = self.x_position.is_some() && self.y_position.is_some() && self.zoom.is_some() && self.size.is_some();
        if ready {
            self.redraw_needed.set();
        }
    }

    fn redraw_needed(&self) -> RedrawNeeded { self.redraw_needed.clone() }
    fn x_position(&self) -> anyhow::Result<f64> { stage_ok(&self.x_position) }
    fn y_position(&self) -> anyhow::Result<f64> { stage_ok(&self.y_position) }
    fn zoom(&self) -> anyhow::Result<f64> { stage_ok(&self.zoom) }
    fn size(&self) -> anyhow::Result<(f64,f64)> { stage_ok(&self.size) }

    fn set_x_position(&mut self, x: f64) { self.x_position = Some(x); self.changed(); }
    fn set_y_position(&mut self, y: f64) { self.y_position = Some(y); self.changed(); }
    fn set_size(&mut self, x: f64, y: f64) { self.size = Some((x,y)); self.changed(); }
    fn set_zoom(&mut self, z: f64) { self.zoom = Some(z); self.changed(); }
}

impl Stage {
    pub fn new() -> Stage { Stage(Rc::new(Mutex::new(StageData::new()))) }
    pub fn redraw_needed(&self) -> RedrawNeeded { self.0.lock().unwrap().redraw_needed() }
    pub fn x_position(&self) -> anyhow::Result<f64> {  self.0.lock().unwrap().x_position() }
    pub fn y_position(&self) -> anyhow::Result<f64> { self.0.lock().unwrap().y_position() }
    pub fn zoom(&self) -> anyhow::Result<f64> { self.0.lock().unwrap().zoom() }
    pub fn size(&self) -> anyhow::Result<(f64,f64)> { self.0.lock().unwrap().size() }
    pub fn set_x_position(&mut self, x: f64) { self.0.lock().unwrap().set_x_position(x); }
    pub fn set_y_position(&mut self, y: f64) { self.0.lock().unwrap().set_y_position(y); }
    pub fn set_size(&mut self, x: f64, y: f64) { self.0.lock().unwrap().set_size(x,y); }
    pub fn set_zoom(&mut self, z: f64) { self.0.lock().unwrap().set_zoom(z); }
}


pub(crate) fn get_stage_source() -> SourceInstrs {
    SourceInstrs::new(vec![
        Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageHpos"),
        Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageVpos"),
        Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageZoom"),
        Uniform::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
        Uniform::new_fragment(PR_LOW,GLArity::Scalar,"uOpacity")
    ])
}
