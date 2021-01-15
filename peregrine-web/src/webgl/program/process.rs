use anyhow::{ anyhow as err, bail };
use std::collections::HashMap;
use crate::webgl::canvas::canvas::Canvas;
use super::program::Program;
use super::attribute::{ Attribute, AttribHandle };
use super::uniform::{ Uniform, UniformHandle };
use super::texture::Texture;
use super::values::{ ProcessValues,  ProcessValueType, AnonProcessValues };
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext, WebGlBuffer, WebGlTexture, HtmlCanvasElement };
use crate::webgl::util::handle_context_errors;

fn create_index_buffer(context: &WebGlRenderingContext, values: &[u16]) -> anyhow::Result<WebGlBuffer> {
    let buffer = context.create_buffer().ok_or(err!("failed to create buffer"))?;
    // After `Int16Array::view` be very careful not to do any memory allocations before it's dropped.
    unsafe {
        let value_array = js_sys::Uint16Array::view(values);
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &value_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        drop(value_array);
    }
    handle_context_errors(context)?;
    Ok(buffer)
}

pub struct Process<'c> {
    program: Program<'c>,
    context: &'c WebGlRenderingContext,
    attribs: ProcessValues<u32,WebGlBuffer,AttribHandle,Vec<f32>>,
    uniforms: ProcessValues<WebGlUniformLocation,Vec<f32>,UniformHandle,Vec<f32>>,
    textures: AnonProcessValues<u32,WebGlTexture,Canvas>,
    index: Option<WebGlBuffer>
}

impl<'c> Process<'c> {
    pub fn new(program: &Program<'c>) -> Process<'c> {
        let mut uniforms = ProcessValues::new();
        let mut attribs = ProcessValues::new();
        for (uniform,location) in program.get_uniforms().iter() {
            uniforms.add_entry(uniform.name(),location.clone(),Box::new(uniform.clone()));
        }
        for (attrib,location) in program.get_attribs().iter() {
            attribs.add_entry(attrib.name(),*location,Box::new(attrib.clone()));
        }
        Process {
            program: program.clone(),
            context: program.context(),
            attribs,
            uniforms,
            textures: AnonProcessValues::new(),
            index: None
        }
    }

    fn activate_index(&self) -> anyhow::Result<()> {
        if let Some(index) = &self.index {
            self.context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,Some(index));
            handle_context_errors(self.context)?;
        }
        Ok(())
    }

    fn drop_index(&self) -> anyhow::Result<()> {
        if let Some(index) = &self.index {
            self.context.delete_buffer(Some(index));
            handle_context_errors(self.context)?;
        }
        Ok(())
    }

    pub fn select_process(&self) -> anyhow::Result<()> {
        self.uniforms.activate_all(&self.context)?;
        self.attribs.activate_all(&self.context)?;
        self.textures.activate_all(&self.context)?;
        self.activate_index()?;
        self.program.select_program()?;
        Ok(())
    }

    pub fn draw(&self) -> anyhow::Result<()> {

        Ok(())
    }

    pub fn get_uniform_handle(&mut self, name: &str) -> anyhow::Result<UniformHandle> {
        self.uniforms.get_handle(name)
    }

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f32>) -> anyhow::Result<()> {
        self.uniforms.set_value(&self.context,handle,values)
    }

    pub fn get_attrib_handle(&mut self, name: &str) -> anyhow::Result<AttribHandle> {
        self.attribs.get_handle(name)
    }

    pub fn set_attrib(&mut self, handle: &AttribHandle, values: Vec<f32>) -> anyhow::Result<()> {
        self.attribs.set_value(&self.context,handle,values)
    }

    pub fn set_index(&mut self, index: &[u16]) -> anyhow::Result<()> {
        self.index = Some(create_index_buffer(&self.context,index)?);
        Ok(())
    }

    pub fn add_texture(&mut self, index: u32, element: &Canvas) -> anyhow::Result<()> {
        self.textures.add_anon(&self.context,Box::new(Texture::new()),index,element.clone())
    }
}

impl<'c> Drop for Process<'c> {
    fn drop(&mut self) {
        self.uniforms.delete(&self.context);
        self.attribs.delete(&self.context);
        self.textures.delete(&self.context);
        self.drop_index();
    }
}
