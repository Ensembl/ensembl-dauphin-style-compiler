use anyhow::{ anyhow as err, bail };
use super::source::Source;
use super::program::Program;
use super::super::{ GLArity, GPUSpec, Precision, Phase };
use web_sys::{ WebGlRenderingContext, WebGlBuffer };
use crate::keyed_handle;
use crate::webgl::util::handle_context_errors;

keyed_handle!(AttribHandle);

#[derive(Clone)]
pub(crate) struct Attribute {
    precision: Precision,
    arity: GLArity,
    name: String,
    location: Option<u32>
}

impl Attribute {
    pub fn new(precision: Precision, arity: GLArity, name: &str) -> Box<Attribute> {
        Box::new(Attribute {
            precision, arity,
            name: name.to_string(),
            location: None
        })
    }

    pub fn name(&self) -> &str { &self.name }
}

impl Source for Attribute {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn declare(&self, spec: &GPUSpec, phase: Phase) -> String {
        if phase != Phase::Vertex { return String::new(); }
        format!("attribute {} {};\n",spec.best_size(&self.precision,&Phase::Vertex).as_string(self.arity),self.name)
    }

    fn build(&mut self, program: &mut Program) -> anyhow::Result<()> { 
        let context = program.context();
        let location = context.get_attrib_location(program.program(),&self.name);
        handle_context_errors(context)?;
        if location == -1 {
            bail!("cannot get attrib '{}'",self.name);
        }
        self.location = Some(location as u32);
        program.add_attrib(&self)
    }
}

fn create_buffer(context: &WebGlRenderingContext, values: &[f64]) -> anyhow::Result<WebGlBuffer> {
    let buffer = context.create_buffer().ok_or(err!("failed to create buffer"))?;
    // After `Float64Array::view` be very careful not to do any memory allocations before it's dropped.
    unsafe {
        let value_array = js_sys::Float64Array::view(values);
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &value_array,
            WebGlRenderingContext::STATIC_DRAW
        );
        drop(value_array);
    }
    handle_context_errors(context)?;
    Ok(buffer)
}

pub(crate) struct AttributeValues {
    gl_value: WebGlBuffer,
    arity: i32,
    location: u32
}

impl AttributeValues {
    pub(super) fn new(object: &Attribute, our_value: Vec<f64>, context: &WebGlRenderingContext) -> anyhow::Result<AttributeValues> {
        Ok(AttributeValues {
            gl_value: create_buffer(context,&our_value)?,
            arity: object.arity.to_num() as i32,
            location: object.location.unwrap()
        })
    }

    pub(super) fn activate(&self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        let location = self.location;
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER,Some(&self.gl_value));
        handle_context_errors(context)?;
        context.enable_vertex_attrib_array(location);
        handle_context_errors(context)?;
        context.vertex_attrib_pointer_with_i32(location,self.arity,WebGlRenderingContext::FLOAT,false,0,0);
        handle_context_errors(context)?;    
        Ok(())
    }

    pub fn discard(&mut self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        context.delete_buffer(Some(&self.gl_value));
        handle_context_errors(context)?;
        Ok(())
    }
}
