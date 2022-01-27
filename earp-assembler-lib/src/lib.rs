pub mod assemble {
    pub mod assembler;
    pub mod lookup;
    pub mod parser;
    pub mod rellabels;    
}

pub mod auxparsers {
    pub mod hexfile;
    pub mod opcodemap;    
}

pub mod core {
    pub mod error;
    pub mod serialize;
    
    #[cfg(test)]
    pub mod testutil;
}

pub mod earpfile {
    pub mod command;
    pub mod earpfile;
    pub mod setmapper;
}

pub mod suite {
    pub mod assets;
    pub mod fileloader;
    pub mod instructionset;
    pub mod suite;
}

pub use crate::assemble::assembler::Assemble;
pub use suite::assets::AssetSource;
pub use crate::core::error::AssemblerError;
pub use suite::fileloader::FileLoader;
pub use crate::earpfile::earpfile::EarpFileWriter;
pub use crate::auxparsers::opcodemap::load_opcode_map;
pub use crate::core::serialize::serialize;
pub use suite::suite::Suite;
