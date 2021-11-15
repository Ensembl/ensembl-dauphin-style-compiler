use std::collections::HashMap;

use super::process::Process;
use crate::shape::layers::layer::ProgramCharacter;
use crate::stage::stage::{ ReadStage };
use peregrine_data::{PeregrineCore, Scale};
use web_sys::{ WebGlRenderingContext };
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

pub struct SessionMetric {
    scale: u64,
    number_of_buffers: usize,
    number_of_processes: usize,
    characters: HashMap<ProgramCharacter,usize>
}

impl SessionMetric {
    fn new(scale: Option<Scale>) -> SessionMetric {
        SessionMetric {
            scale: scale.map(|s| s.get_index()).unwrap_or(0),
            number_of_buffers: 0,
            number_of_processes: 0,
            characters: HashMap::new()
        }
    }

    pub(crate) fn add_character(&mut self, ch: &ProgramCharacter) {
        *self.characters.entry(ch.clone()).or_insert(0) += 1;
    }

    fn add_process(&mut self, buffers: usize) {
        self.number_of_buffers += buffers;
        self.number_of_processes += 1;
    }

    fn send_metric(&self, core: &PeregrineCore) {
        core.general_metric("gb-render",vec![
            ("scale".to_string(),self.scale.to_string())
        ],vec![
            ("buffers".to_string(),self.number_of_buffers as f64),
            ("processes".to_string(),self.number_of_processes as f64),
            ("characters".to_string(),self.characters.len() as f64)
        ]);
        let ch = self.characters.iter().map(|(ch,value)| {
            (ch.key().replace(|c: char| !c.is_alphanumeric(),""),*value as f64)
        }).collect::<Vec<_>>();
        core.general_metric("gb-characters",vec![
            ("scale".to_string(),self.scale.to_string())
        ],ch);
    }
}

pub(crate) struct DrawingSession {
    metric: SessionMetric,
}

impl DrawingSession {
    pub fn new(scale: Option<Scale>) -> DrawingSession {
        DrawingSession {
            metric: SessionMetric::new(scale)
        }
    }

    pub(crate) fn metric(&mut self) -> &mut SessionMetric { &mut self.metric }

    pub(crate) fn run_process(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, process: &mut Process, opacity: f64) -> Result<(),Message> {
        self.metric.add_process(process.number_of_buffers());
        process.draw(gl,stage,opacity,&mut self.metric)
    }

    pub(crate) fn begin(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let size = gl.canvas_size().clone()
            .ok_or_else(|| Message::ConfusedWebBrowser(format!("unsized canvas")))?;
        //use web_sys::console;
        //console::log_1(&format!("init {} {}",size.0,size.1).into());    
        gl.context().enable(WebGlRenderingContext::DEPTH_TEST);
        gl.context().enable(WebGlRenderingContext::BLEND);
        gl.context().enable(WebGlRenderingContext::SCISSOR_TEST);
        gl.context().depth_func(WebGlRenderingContext::LEQUAL);
        gl.context().viewport(0,0,size.0 as i32,size.1 as i32);
        gl.context().scissor(0,0,size.0 as i32,size.1 as i32);
        gl.context().clear_color(1., 1., 1., 1.);
        gl.context().depth_mask(true);
        gl.handle_context_errors()?;
        gl.context().clear(WebGlRenderingContext::COLOR_BUFFER_BIT|WebGlRenderingContext::DEPTH_BUFFER_BIT);
        gl.handle_context_errors()?;
        gl.context().blend_func_separate(WebGlRenderingContext::SRC_ALPHA, WebGlRenderingContext::ONE_MINUS_SRC_ALPHA, WebGlRenderingContext::ONE, WebGlRenderingContext::ONE_MINUS_SRC_ALPHA);
        gl.handle_context_errors()?;
        Ok(())
    }

    pub(crate) fn finish(&self, core: &PeregrineCore) -> Result<(),Message> {
        self.metric.send_metric(core);
        Ok(())
    }
}
