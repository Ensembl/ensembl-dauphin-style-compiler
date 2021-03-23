use super::process::Process;
use crate::shape::core::stage::{ Stage, ReadStage, ReadStageAxis };
use web_sys::{ WebGlRenderingContext };
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

// TODO clever viewport on resize

pub struct DrawingSession {
    size: Option<(f64,f64)>
}

impl DrawingSession {
    pub fn new() -> DrawingSession {
        DrawingSession {
           size: None
        }
    }

    fn update_viewport(&mut self, gl: &mut WebGlGlobal,  stage: &ReadStage) -> Result<(),Message> {
        let size = (stage.x().size()?,stage.y().size()?);
        if let Some((old_x,old_y)) = self.size {
            if old_x == size.0 && old_y == size.1 {
                return Ok(())
            }
        }
        self.size = Some(size);
        gl.context().viewport(0,0,size.0 as i32,size.1 as i32);
        gl.handle_context_errors()?;
        Ok(())
    }

    pub(crate) fn run_process(&self, gl: &mut WebGlGlobal, stage: &ReadStage, process: &mut Process, opacity: f64) -> Result<(),Message> {
        process.draw(gl,stage,opacity)
    }

    pub(crate) fn begin(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage) -> Result<(),Message> {
        self.update_viewport(gl,stage)?;
        gl.context().clear_color(1., 1., 1., 1.);
        gl.handle_context_errors()?;
        gl.context().clear(WebGlRenderingContext::COLOR_BUFFER_BIT|WebGlRenderingContext::DEPTH_BUFFER_BIT);
        gl.handle_context_errors()?;
        Ok(())
    }

    pub(crate) fn finish(&self) -> Result<(),Message> {
        Ok(())
    }
}