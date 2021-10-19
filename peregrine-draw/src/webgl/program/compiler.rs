use crate::{ webgl::{ SourceInstrs, Phase, GPUSpec }};
use web_sys::{ WebGlRenderingContext, WebGlShader, WebGlProgram };
use crate::webgl::util::handle_context_errors;
use crate::util::message::Message;

fn compile_shader(context: &WebGlRenderingContext, shader_type: u32, source: &str) -> Result<WebGlShader,Message> {
    let shader = context.create_shader(shader_type).ok_or_else(|| Message::WebGLFailure("Unable to create shader object".to_string()))?;
    handle_context_errors(context)?;
    context.shader_source(&shader, source);
    handle_context_errors(context)?;
    context.compile_shader(&shader);
    handle_context_errors(context)?;
    if context.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false)
    {
        handle_context_errors(context)?;
        Ok(shader)
    } else {
        Err(Message::WebGLFailure(context.get_shader_info_log(&shader).unwrap_or_else(|| String::from("Unknown error creating shader"))))
    }
}

fn make_vertex_shader(context: &WebGlRenderingContext, gpuspec: &GPUSpec, source: &SourceInstrs) -> Result<WebGlShader,Message> {
    let source_text = source.serialise(gpuspec,Phase::Vertex);
    compile_shader(context,WebGlRenderingContext::VERTEX_SHADER,&source_text)
}

fn make_fragment_shader(context: &WebGlRenderingContext, gpuspec: &GPUSpec, source: &SourceInstrs) -> Result<WebGlShader,Message> {
    let source_text = source.serialise(gpuspec,Phase::Fragment);
    compile_shader(context,WebGlRenderingContext::FRAGMENT_SHADER,&source_text)
}

pub(crate) fn make_program(context: &WebGlRenderingContext, gpuspec: &GPUSpec, source: SourceInstrs) -> Result<WebGlProgram,Message> {
    let program = context.create_program().ok_or_else(|| Message::WebGLFailure(format!("could not create program")))?;
    handle_context_errors(&context)?;
    context.attach_shader(&program,&make_vertex_shader(context,gpuspec,&source)?);
    handle_context_errors(&context)?;
    context.attach_shader(&program,&make_fragment_shader(context,gpuspec,&source)?);
    handle_context_errors(&context)?;
    context.link_program(&program);
    handle_context_errors(&context)?;
    if !context.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS).as_bool().unwrap_or(false) {
        return Err(Message::WebGLFailure(context.get_program_info_log(&program).unwrap_or_else(|| String::from("Unknown error creating program object"))))
    }
    handle_context_errors(&context)?;
    Ok(program)
}
