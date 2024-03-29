use crate::{ webgl::{ SourceInstrs, Phase, GPUSpec, util::handle_context_errors2 }};
use peregrine_toolkit::error::Error;
use web_sys::{ WebGlRenderingContext, WebGlShader, WebGlProgram };

#[cfg(not(debug_assertions))]
fn check_shader(_context: &WebGlRenderingContext, _shader: &WebGlShader) -> Result<(),Error> {
    Ok(())
}

#[cfg(debug_assertions)]
fn check_shader(context: &WebGlRenderingContext, shader: &WebGlShader) -> Result<(),Error> {
    if context.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false)
    {
        handle_context_errors2(context)?;
        Ok(())
    } else {
        Err(Error::fatal(&context.get_shader_info_log(&shader).unwrap_or_else(|| String::from("Unknown error creating shader"))))
    }
}

#[cfg(not(debug_assertions))]
fn check_program(context: &WebGlRenderingContext, program: &WebGlProgram) -> Result<(),Error> {
    Ok(())
}

#[cfg(debug_assertions)]
fn check_program(context: &WebGlRenderingContext, program: &WebGlProgram) -> Result<(),Error> {
    if !context.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS).as_bool().unwrap_or(false) {
        Err(Error::fatal(&context.get_program_info_log(&program).unwrap_or_else(|| String::from("Unknown error creating program object"))))
    } else {
        Ok(())
    }
}

fn compile_shader(context: &WebGlRenderingContext, shader_type: u32, source: &str) -> Result<WebGlShader,Error> {
    let shader = context.create_shader(shader_type).ok_or_else(|| Error::fatal("Unable to create shader object"))?;
    handle_context_errors2(context)?;
    context.shader_source(&shader,source);
    handle_context_errors2(context)?;
    context.compile_shader(&shader);
    handle_context_errors2(context)?;
    check_shader(context,&shader)?;
    Ok(shader)
}

fn make_vertex_shader(context: &WebGlRenderingContext, gpuspec: &GPUSpec, source: &SourceInstrs) -> Result<WebGlShader,Error> {
    let source_text = source.serialise(gpuspec,Phase::Vertex);
    compile_shader(context,WebGlRenderingContext::VERTEX_SHADER,&source_text)
}

fn make_fragment_shader(context: &WebGlRenderingContext, gpuspec: &GPUSpec, source: &SourceInstrs) -> Result<WebGlShader,Error> {
    let source_text = source.serialise(gpuspec,Phase::Fragment);
    compile_shader(context,WebGlRenderingContext::FRAGMENT_SHADER,&source_text)
}

pub(crate) fn make_program(context: &WebGlRenderingContext, gpuspec: &GPUSpec, source: SourceInstrs) -> Result<WebGlProgram,Error> {
    let program = context.create_program().ok_or_else(|| Error::fatal("could not create program"))?;
    handle_context_errors2(&context)?;
    context.attach_shader(&program,&make_vertex_shader(context,gpuspec,&source)?);
    handle_context_errors2(&context)?;
    context.attach_shader(&program,&make_fragment_shader(context,gpuspec,&source)?);
    handle_context_errors2(&context)?;
    context.link_program(&program);
    handle_context_errors2(&context)?;
    check_program(&context,&program)?;
    handle_context_errors2(&context)?;
    Ok(program)
}
