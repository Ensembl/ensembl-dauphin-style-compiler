use anyhow::{ anyhow as err };
use super::source::Source;
use super::program::Program;
use super::super::{ GLArity, GPUSpec, Precision, Phase };
use web_sys::{ WebGlRenderingContext, WebGlBuffer };
use super::values::ProcessValueType;
use crate::process_value_handle;

process_value_handle!(AttribHandle);

#[derive(Clone)]
pub(crate) struct Attribute {
    precision: Precision,
    arity: GLArity,
    name: String
}

fn create_buffer(context: &WebGlRenderingContext, values: &[f32]) -> anyhow::Result<WebGlBuffer> {
    let buffer = context.create_buffer().ok_or(err!("failed to create buffer"))?;
    // After `Float32Array::view` be very careful not to do any memory allocations before it's dropped.
    unsafe {
        let value_array = js_sys::Float32Array::view(values);
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &value_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        drop(value_array);
    }
    Ok(buffer)
}

impl Attribute {
    pub fn new(precision: Precision, arity: GLArity, name: &str) -> Box<Attribute> {
        Box::new(Attribute {
            precision, arity,
            name: name.to_string(),
        })
    }
}

impl Source for Attribute {
    fn declare(&self, spec: &GPUSpec, phase: Phase) -> String {
        if phase != Phase::Vertex { return String::new(); }
        format!("attribute {} {};\n",spec.best_size(&self.precision,&Phase::Vertex).as_string(self.arity),self.name)
    }

    fn build(&self, program: &mut Program) -> anyhow::Result<()> {
        program.add_attrib(&self)
    }
}

impl ProcessValueType for Attribute {
    type OurValue = Vec<f32>;
    type GLKey = u32;
    type GLValue = WebGlBuffer;

    fn name(&self) -> &str { &self.name }

    fn activate(&self, context: &WebGlRenderingContext, gl_key: &u32, gl_value: &WebGlBuffer) -> anyhow::Result<()> {
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER,Some(gl_value));
        context.enable_vertex_attrib_array(*gl_key);
        context.vertex_attrib_pointer_with_i32(*gl_key,self.arity.to_num() as i32,WebGlRenderingContext::FLOAT,false,0,0);
        Ok(())
    }

    fn value_to_gl(&self, context: &WebGlRenderingContext, our_data: Self::OurValue)  -> anyhow::Result<Self::GLValue> {
        create_buffer(context,&our_data)
    }

    fn delete(&self, context: &WebGlRenderingContext, gl_value: &Self::GLValue) -> anyhow::Result<()> {
        context.delete_buffer(Some(gl_value));
        Ok(())
    }
}
