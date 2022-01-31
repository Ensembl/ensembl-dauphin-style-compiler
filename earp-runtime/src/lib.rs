pub mod runtime {
    pub mod command;
    pub mod context;
    pub mod instruction;
    pub mod operand;
}

pub mod core {
    pub mod error;
}

pub mod earpfile {
    pub mod toplevel;
    pub mod earpfilereader;
    pub mod resolver;
}

pub mod suite {
    pub mod instructionset;
    pub mod suite;
}

mod registerfile;

pub use crate::registerfile::{ EarpFunction, EarpStatement, EarpArgument, EarpProgram, EarpRuntime, EarpReturn, EarpOutcome };
