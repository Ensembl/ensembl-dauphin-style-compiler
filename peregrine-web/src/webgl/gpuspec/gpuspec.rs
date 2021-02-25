use anyhow::{ anyhow as err, bail };
use std::cmp::min;
use super::precision::Precision;
use super::glsize::GLSize;
use web_sys::{ WebGlRenderingContext, WebGlShaderPrecisionFormat };

#[derive(Clone,Copy,PartialEq,Eq)]
pub(crate) enum Phase {
    Vertex,
    Fragment
}

fn match_precision(precision: WebGlShaderPrecisionFormat, is_int: bool) -> Precision {
    let range = min(precision.range_min(),precision.range_max());
    if is_int {
        Precision::Int(range)
    } else {
        Precision::Float(range,precision.precision())
    }
}

fn get_precision(context: &WebGlRenderingContext, shader: u32, gltype: u32, is_int: bool) -> Option<Precision> {
    context.get_shader_precision_format(shader,gltype).map(|p| match_precision(p,is_int))
}

fn add_precision(out: &mut Vec<(GLSize,Precision)>, context: &WebGlRenderingContext, shader: u32, glsize: GLSize) {
    if let Some(gltype) = get_precision(context,shader,glsize.get_gltype(),glsize.is_int()) {
        out.push((glsize,gltype));
    }
}

fn get_precisions(out: &mut Vec<(GLSize,Precision)>, context: &WebGlRenderingContext, shader: u32) {
    add_precision(out,context,shader,GLSize::IntLow);
    add_precision(out,context,shader,GLSize::IntMed);
    add_precision(out,context,shader,GLSize::IntHigh);
    add_precision(out,context,shader,GLSize::FloatLow);
    add_precision(out,context,shader,GLSize::FloatMed);
    add_precision(out,context,shader,GLSize::FloatHigh);
}

fn get_parameter_u32(context: &WebGlRenderingContext, name: u32) -> anyhow::Result<u32> {
    let value : Option<f64> = context.get_parameter(name).map_err(|e| err!("could not get {}: {:?}",name,e.as_string()))?.as_f64();
    let value = value.ok_or_else(|| err!("could not get {}: null value",name))?;
    Ok(value as u32)
}

fn best_size(want: &Precision, sizes: &Vec<(GLSize,Precision)>) -> GLSize {
    for (size,precision) in sizes {
        if precision >= want {
            return *size;
        }
    }
    return sizes[sizes.len()-1].0
}

#[derive(Clone)]
pub(crate) struct GPUSpec {
    vert_precs: Vec<(GLSize,Precision)>,
    frag_precs: Vec<(GLSize,Precision)>,
    max_texture_size: u32,
    max_textures: u32
}

impl GPUSpec {
    pub fn new(context: &WebGlRenderingContext) -> anyhow::Result<GPUSpec> { 
        let mut out = GPUSpec {
            vert_precs: Vec::new(),
            frag_precs: Vec::new(),
            max_texture_size: get_parameter_u32(context,WebGlRenderingContext::MAX_TEXTURE_SIZE)?,
            max_textures: get_parameter_u32(context,WebGlRenderingContext::MAX_TEXTURE_IMAGE_UNITS)?
        };
        out.populate(context)?;
        Ok(out)
    }

    fn populate(&mut self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        get_precisions(&mut self.vert_precs,context,WebGlRenderingContext::VERTEX_SHADER);
        get_precisions(&mut self.frag_precs,context,WebGlRenderingContext::FRAGMENT_SHADER);
        if self.vert_precs.len() == 0 || self.frag_precs.len() == 0 {
            bail!("retrieving GPU spec failed")
        }
        Ok(())
    }

    pub fn best_size(&self, want: &Precision, phase: &Phase) -> GLSize {
        let var = match phase {
            Phase::Vertex => &self.vert_precs,
            Phase::Fragment => &self.frag_precs
        };
        best_size(want,var)
    }

    pub fn max_texture_size(&self) -> u32 { self.max_texture_size }
    pub fn max_textures(&self) -> u32 { self.max_textures }
}
