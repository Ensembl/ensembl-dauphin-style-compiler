use crate::webgl::{ SourceInstrs, WebGlCompiler, Program };
use super::paintgeometry::PaintGeometry;
use super::paintskin::PaintSkin;
use super::paintmethod::PaintMethod;
use web_sys::{ WebGlProgram };

fn make_source(method: PaintMethod, geometry: PaintGeometry, skin: PaintSkin) -> SourceInstrs {
    let mut program = SourceInstrs::new(vec![]);
    program.merge(method.to_source());
    program.merge(geometry.to_source());
    program.merge(skin.to_source());
    program
}

pub struct Layer<'c> {
    program: Program<'c>
}

impl<'c> Layer<'c> {
    pub(crate) fn new(compiler: &WebGlCompiler<'c>, method: PaintMethod, geometry: PaintGeometry, skin: PaintSkin) -> anyhow::Result<Layer<'c>> {
        let source = make_source(method,geometry,skin);
        Ok(Layer {
            program: compiler.make_program(source)?
        })
    }
}