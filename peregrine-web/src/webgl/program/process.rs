use anyhow::{ anyhow as err, bail };
use std::collections::HashMap;
use std::rc::Rc;
use crate::webgl::canvas::canvas::Canvas;
use super::program::Program;
use super::attribute::{ Attribute, AttribHandle, AttributeValues };
use super::uniform::{ Uniform, UniformHandle, UniformValues };
use super::texture::{ Texture, TextureValues };
use super::keyed::{ KeyedValues };
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
    program: Rc<Program<'c>>,
    context: &'c WebGlRenderingContext,
    attribs: KeyedValues<AttribHandle,AttributeValues>,
    uniforms: KeyedValues<UniformHandle,UniformValues>,
    textures: Vec<TextureValues>,
    index: Option<WebGlBuffer>,
    len: usize
}

impl<'c> Process<'c> {
    pub(super) fn new(program: Rc<Program<'c>>, context: &'c WebGlRenderingContext) -> Process<'c> {
        let mut uniforms = KeyedValues::new();
        let mut attribs = KeyedValues::new();
        for uniform in program.get_uniforms().iter() {
            uniforms.add(uniform.name(),UniformValues::new(uniform.clone()));
        }
        for attrib in program.get_attribs().iter() {
            attribs.add(attrib.name(),AttributeValues::new(attrib.clone()));
        }        
        Process {
            program, context,
            attribs,
            uniforms,
            textures: vec![],
            index: None,
            len: 0
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

    pub fn draw(&self) -> anyhow::Result<()> {
        for entry in self.uniforms.iter() {
            entry.activate(&self.context)?;
        }
        for entry in self.attribs.iter() {
            entry.activate(&self.context)?;
        }
        for entry in self.textures.iter() {
            entry.activate(&self.context)?;
        }
        self.activate_index()?;
        self.program.select_program()?;
        self.context.draw_elements_with_i32(self.program.get_method(),self.len as i32,WebGlRenderingContext::UNSIGNED_SHORT,0);
        handle_context_errors(self.context)?;
        Ok(())
    }

    pub fn get_uniform_handle(&mut self, name: &str) -> anyhow::Result<UniformHandle> {
        self.uniforms.get_handle(name)
    }

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f32>) -> anyhow::Result<()> {
        self.uniforms.get_mut(handle).set_value(&self.context,values)
    }

    pub fn get_attrib_handle(&mut self, name: &str) -> anyhow::Result<AttribHandle> {
        self.attribs.get_handle(name)
    }

    pub fn set_attrib(&mut self, handle: &AttribHandle, values: Vec<f32>) -> anyhow::Result<()> {
        self.attribs.get_mut(handle).set_value(&self.context,values)
    }

    pub fn set_index(&mut self, index: &[u16]) -> anyhow::Result<()> {
        self.index = Some(create_index_buffer(&self.context,index)?);
        self.len = index.len();
        Ok(())
    }

    pub fn add_texture(&mut self, index: u32, element: &Canvas) -> anyhow::Result<()> {
        let mut entry = TextureValues::new(&self.context,index,element.clone())?;
        self.textures.push(entry);
        Ok(())
    }
}

impl<'c> Drop for Process<'c> {
    fn drop(&mut self) {
        for entry in self.attribs.iter_mut() {
            entry.delete(&self.context);
        }
        for entry in self.textures.iter_mut() {
            entry.delete(&self.context);
        }
        self.drop_index();
    }
}
