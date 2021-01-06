use std::cmp::min;
use super::precision::Precision;
use super::glsize::GLSize;
use web_sys::{ WebGlRenderingContext, WebGlShaderPrecisionFormat };

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

fn best_size(want: &Precision, sizes: &Vec<(GLSize,Precision)>) -> GLSize {
    for (size,precision) in sizes {
        if precision >= want {
            return *size;
        }
    }
    return sizes[sizes.len()-1].0
}

struct GPUSpec {
    vert_precs: Vec<(GLSize,Precision)>,
    frag_precs: Vec<(GLSize,Precision)>
}

impl GPUSpec {
    pub fn new() -> GPUSpec { 
        GPUSpec {
            vert_precs: Vec::new(),
            frag_precs: Vec::new()
        }
    }

    pub fn populate(&mut self, context: &WebGlRenderingContext) {
        get_precisions(&mut self.vert_precs,context,WebGlRenderingContext::VERTEX_SHADER);
        get_precisions(&mut self.vert_precs,context,WebGlRenderingContext::FRAGMENT_SHADER);
    }

    pub fn best_vertex_size(&self, want: &Precision) -> GLSize {
        best_size(want,&self.vert_precs)
    }

    pub fn best_fragment_size(&self, want: &Precision) -> GLSize {
        best_size(want,&self.frag_precs)
    }
}
