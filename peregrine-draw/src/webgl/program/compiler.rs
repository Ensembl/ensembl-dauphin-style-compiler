use anyhow::{ anyhow as err };
use crate::{ webgl::{ SourceInstrs, Phase, GPUSpec }};
use super::program::Program;
use web_sys::{ WebGlRenderingContext, WebGlShader };
use crate::webgl::util::handle_context_errors;
use crate::util::message::Message;

pub(crate) struct WebGlCompiler {
    context: WebGlRenderingContext,
    gpuspec: GPUSpec
}

impl WebGlCompiler {
    pub(crate) fn new(context: &WebGlRenderingContext, gpuspec: &GPUSpec) -> WebGlCompiler {
        WebGlCompiler {
            context:context.clone(),
            gpuspec: gpuspec.clone()
        }
    }

    fn compile_shader(&self, shader_type: u32, source: &str) -> Result<WebGlShader,Message> {
        let shader = self.context.create_shader(shader_type).ok_or_else(|| Message::WebGLFailure("Unable to create shader object".to_string()))?;
        handle_context_errors(&self.context)?;
        self.context.shader_source(&shader, source);
        handle_context_errors(&self.context)?;
        self.context.compile_shader(&shader);
        handle_context_errors(&self.context)?;
        if self.context.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false)
        {
            handle_context_errors(&self.context)?;
            Ok(shader)
        } else {
            Err(Message::WebGLFailure(self.context.get_shader_info_log(&shader).unwrap_or_else(|| String::from("Unknown error creating shader"))))
        }
    }
    
    fn make_vertex_shader(&self, source: &SourceInstrs) -> Result<WebGlShader,Message> {
        let source_text = source.serialise(&self.gpuspec,Phase::Vertex);
        self.compile_shader(WebGlRenderingContext::VERTEX_SHADER,&source_text)
    }
    
    fn make_fragment_shader(&self, source: &SourceInstrs) -> Result<WebGlShader,Message> {
        let source_text = source.serialise(&self.gpuspec,Phase::Fragment);
        self.compile_shader(WebGlRenderingContext::FRAGMENT_SHADER,&source_text)
    }
    
    pub(crate) fn make_program(&self, source: SourceInstrs) -> Result<Program,Message> {
        let program = self.context.create_program().ok_or_else(|| Message::WebGLFailure(format!("could not create program")))?;
        handle_context_errors(&self.context)?;
        self.context.attach_shader(&program,&self.make_vertex_shader(&source)?);
        handle_context_errors(&self.context)?;
        self.context.attach_shader(&program,&self.make_fragment_shader(&source)?);
        handle_context_errors(&self.context)?;
        self.context.link_program(&program);
        handle_context_errors(&self.context)?;
        if self.context.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS).as_bool().unwrap_or(false) {
            handle_context_errors(&self.context)?;
            Ok(Program::new(&self.context,program,source)?)
        } else {
            Err(Message::WebGLFailure(self.context.get_program_info_log(&program).unwrap_or_else(|| String::from("Unknown error creating program object"))))
        }
    }    
}
