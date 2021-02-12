use super::process::Process;
use crate::shape::core::stage::Stage;
use web_sys::{ WebGlRenderingContext };
use crate::webgl::util::handle_context_errors;

// TODO clever viewport on resize

pub struct DrawingSession {
    stage: Stage,
    context: WebGlRenderingContext,
    size: Option<(f64,f64)>
}

impl DrawingSession {
    pub fn new(context: &WebGlRenderingContext, stage: &Stage) -> DrawingSession {
        DrawingSession {
           stage: stage.clone(),
           context: context.clone(),
           size: None
        }
    }

    fn update_viewport(&mut self) -> anyhow::Result<()> {
        let size = self.stage.size()?;
        if let Some((old_x,old_y)) = self.size {
            if old_x == size.0 && old_y == size.1 {
                return Ok(())
            }
        }
        self.size = Some(size);
        self.context.viewport(0,0,size.0 as i32,size.1 as i32);
        handle_context_errors(&self.context)?;
        Ok(())
    }

    pub(crate) fn run_process(&self, process: &mut Process, opacity: f64) -> anyhow::Result<()> {
        process.draw(&self.stage,opacity)
    }

    pub(crate) fn begin(&mut self) -> anyhow::Result<()> {
        self.update_viewport()?;
        self.context.clear_color(1., 1., 1., 1.);
        handle_context_errors(&self.context)?;
        self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT|WebGlRenderingContext::DEPTH_BUFFER_BIT);
        handle_context_errors(&self.context)?;
        Ok(())
    }

    pub(crate) fn finish(&self) -> anyhow::Result<()> {
        Ok(())
    }
}