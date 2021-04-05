use super::process::Process;
use crate::shape::core::stage::{ ReadStage };
use web_sys::{ WebGlRenderingContext };
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

// TODO clever viewport on resize

pub(crate) struct DrawingSession {
}

impl DrawingSession {
    pub fn new() -> DrawingSession {
        DrawingSession {
        }
    }

    pub(crate) fn run_process(&self, gl: &mut WebGlGlobal, stage: &ReadStage, process: &mut Process, opacity: f64) -> Result<(),Message> {
        process.draw(gl,stage,opacity)
    }

    pub(crate) fn begin(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let size = gl.canvas_size().clone()
            .ok_or_else(|| Message::ConfusedWebBrowser(format!("unsized canvas")))?;
        //use web_sys::console;
        //console::log_1(&format!("init {} {}",size.0,size.1).into());    
        gl.context().enable(WebGlRenderingContext::SCISSOR_TEST);
        gl.context().viewport(0,0,size.0 as i32,size.1 as i32);
        gl.context().scissor(0,0,size.0 as i32,size.1 as i32);
        gl.context().clear_color(1., 1., 1., 1.);
        gl.context().enable(WebGlRenderingContext::DEPTH_TEST);
        gl.handle_context_errors()?;
        gl.context().clear(WebGlRenderingContext::COLOR_BUFFER_BIT|WebGlRenderingContext::DEPTH_BUFFER_BIT);
        gl.handle_context_errors()?;
        Ok(())
    }

    pub(crate) fn finish(&self) -> Result<(),Message> {
        Ok(())
    }
}