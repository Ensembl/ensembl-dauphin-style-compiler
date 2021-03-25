use anyhow::{ anyhow as err };
use std::rc::Rc;
use std::cell::RefCell;
use std::future::Future;
use std::sync::Mutex;
use crate::webgl::{ SourceInstrs, Uniform, GLArity, UniformHandle, Program, Process };
use commander::PromiseFuture;
use super::super::layers::consts::{ PR_DEF, PR_LOW };
use super::redrawneeded::RedrawNeeded;
use crate::util::message::Message;

#[derive(Clone)]
struct BootLock(Boot,bool);

impl BootLock {
    fn unlock(&mut self) {
        if self.1 {
            self.1 = false;
            self.0.unlock();
        }
    }
}

#[derive(Clone)]
struct Boot(Rc<RefCell<usize>>);

impl Boot {
    fn new() -> Boot {
        Boot(Rc::new(RefCell::new(0)))
    }

    fn lock(&self) -> BootLock {
        *self.0.borrow_mut() += 1;
        BootLock(self.clone(),true)
    }

    fn unlock(&self) {
        *self.0.borrow_mut() -= 1;
    }

    fn booted(&self) -> bool {
        *self.0.borrow() == 0
    }
}

#[derive(Clone)]
pub(crate) struct ProgramStage {
    hpos: UniformHandle,
    vpos: UniformHandle,
    bp_per_screen: UniformHandle,
    size: UniformHandle,
    opacity: UniformHandle
}

impl ProgramStage {
    pub fn new(program: &Rc<Program>) -> Result<ProgramStage,Message> {
        Ok(ProgramStage {
            hpos: program.get_uniform_handle("uStageHpos")?,
            vpos: program.get_uniform_handle("uStageVpos")?,
            bp_per_screen: program.get_uniform_handle("uStageZoom")?,
            size: program.get_uniform_handle("uSize")?,
            opacity: program.get_uniform_handle("uOpacity")?
        })
    }

    pub fn apply(&self, stage: &ReadStage, left: f64, opacity: f64, process: &mut Process) -> Result<(),Message> {
        process.set_uniform(&self.hpos,vec![stage.x.position()?-left])?;
        process.set_uniform(&self.vpos,vec![stage.y.position()?])?;
        process.set_uniform(&self.bp_per_screen,vec![2./stage.x.bp_per_screen()?])?;
        let size = (stage.x.size()?,stage.y.size()?);
        process.set_uniform(&self.size,vec![size.0/2.,size.1/2.])?;
        process.set_uniform(&self.opacity,vec![opacity])?;
        Ok(())
    }
}

fn stage_ok<T: Clone>(x: &Option<T>) -> Result<T,Message> {
    x.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed("accseeor used on non-ready stage".to_string()))
}

pub trait ReadStageAxis {
    fn position(&self) -> Result<f64,Message>;
    fn bp_per_screen(&self) -> Result<f64,Message>;
    fn size(&self) -> Result<f64,Message>;   
    fn copy(&self) -> StageAxis;
    fn version(&self) -> u64;
    fn ready(&self) -> bool;
}

pub struct StageAxis {
    position: Option<f64>,
    bp_per_screen: Option<f64>,
    size: Option<f64>,
    redraw_needed: RedrawNeeded,
    boot: Boot,
    boot_lock: BootLock,
    version: u64
}

impl StageAxis {
    fn new(redraw_needed: &RedrawNeeded) -> StageAxis {
        let boot = Boot::new();
        let boot_lock = boot.lock();
        StageAxis {
            position: None,
            bp_per_screen: None,
            size: None,
            redraw_needed: redraw_needed.clone(),
            boot: boot.clone(),
            boot_lock,
            version: 0
        }
    }

    fn data_ready(&self) -> bool {
        self.position.is_some() && self.bp_per_screen.is_some()
    }

    fn changed(&mut self) {
        if !self.boot.booted() {
            if self.data_ready() {
                self.boot_lock.unlock();
            }
        }
        if self.boot.booted() {
            self.redraw_needed.set();
        }
        self.version += 1;
    }

    pub fn set_position(&mut self, x: f64) { self.position = Some(x); self.changed(); }
    pub fn set_size(&mut self, x: f64) { self.size = Some(x); self.changed(); }
    pub fn set_bp_per_screen(&mut self, z: f64) { self.bp_per_screen = Some(z); self.changed(); }
}

impl ReadStageAxis for StageAxis {
    fn position(&self) -> Result<f64,Message> { stage_ok(&self.position) }
    fn bp_per_screen(&self) -> Result<f64,Message> { stage_ok(&self.bp_per_screen) }
    fn size(&self) -> Result<f64,Message> { stage_ok(&self.size) }

    // secret clone only accessible via read-only subsets
    fn copy(&self) -> StageAxis {
        StageAxis {
            position: self.position.clone(),
            bp_per_screen: self.bp_per_screen.clone(),
            size: self.size.clone(),
            redraw_needed: self.redraw_needed.clone(),
            version: self.version,
            boot: self.boot.clone(),
            boot_lock: self.boot_lock.clone()
        }
    }    

    fn ready(&self) -> bool {
        self.position.is_some() && self.bp_per_screen.is_some() && self.size.is_some()
    }

    fn version(&self) -> u64 { self.version }
}

// TODO greedy canvas size changes
pub struct Stage {
    x: StageAxis,
    y: StageAxis,
    redraw_needed: RedrawNeeded
}

pub struct ReadStage {
    x: Box<dyn ReadStageAxis>,
    y: Box<dyn ReadStageAxis>    
}

impl ReadStage {
    pub fn x(&self) -> &dyn ReadStageAxis { self.x.as_ref() }
    pub fn y(&self) -> &dyn ReadStageAxis { self.y.as_ref() }
    pub fn ready(&self) -> bool { self.x.ready() && self.y.ready() }
}

impl Clone for ReadStage {
    fn clone(&self) -> Self {
        ReadStage {
            x: Box::new(self.x.copy()),
            y: Box::new(self.y.copy())
        }
    }
}

impl Stage {
    pub fn new() -> Stage {
        let redraw_needed = RedrawNeeded::new();
        let data_needed = RedrawNeeded::new();
        let mut out = Stage {
            x: StageAxis::new(&redraw_needed),
            y: StageAxis::new(&redraw_needed),
            redraw_needed
        };
        out.y.set_position(0.);
        out.y.set_bp_per_screen(1.);
        out
    }

    pub fn ready(&self) -> bool { self.x.ready() && self.y.ready() }

    pub fn redraw_needed(&self) -> RedrawNeeded { self.redraw_needed.clone() }

    pub fn x(&self) -> &StageAxis { &self.x }
    pub fn y(&self) -> &StageAxis { &self.y }
    pub fn x_mut(&mut self) -> &mut StageAxis { &mut self.x }
    pub fn y_mut(&mut self) -> &mut StageAxis { &mut self.y }

    pub fn read_stage(&self) -> ReadStage {
        ReadStage {
            x: Box::new(self.x.copy()),
            y: Box::new(self.y.copy())
        }
    }
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
