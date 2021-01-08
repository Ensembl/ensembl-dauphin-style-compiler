use crate::webgl::Program;
use super::paintgeometry::PaintGeometry;
use super::paintskin::PaintSkin;
use super::paintmethod::PaintMethod;

fn make_program(method: PaintMethod, geometry: PaintGeometry, skin: PaintSkin) -> Program {
    let mut program = Program::new(vec![]);
    program.merge(method.to_source());
    program.merge(geometry.to_source());
    program.merge(skin.to_source());
    program
}

pub struct Layer {
    program: Program
}

impl Layer {
    pub(crate) fn new(method: PaintMethod, geometry: PaintGeometry, skin: PaintSkin) -> Layer {
        Layer {
            program: make_program(method,geometry,skin)
        }
    }
}