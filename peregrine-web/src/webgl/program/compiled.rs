use web_sys::{ WebGlProgram };

pub struct Compiled {
    program: WebGlProgram
}

impl Compiled {
    pub fn new(program: WebGlProgram) -> Compiled {
        Compiled {
            program
        }
    }
}