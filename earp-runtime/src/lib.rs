pub mod core {
    pub mod error;
}

pub mod earpfile {
    pub mod toplevel;
    pub mod earpfilereader;
}

mod registerfile;

pub use crate::registerfile::{ EarpFunction, EarpStatement, EarpArgument, EarpProgram, EarpRuntime, EarpReturn, EarpOutcome };
