mod assemble;
mod assets;
mod command;
mod earpfile;
mod error;
mod fileloader;
mod hexfile;
mod instructionset;
mod lookup;
mod opcodemap;
mod options;
mod parser;
mod rellabels;
mod setmapper;
mod suite;
#[cfg(test)]
mod testutil;

use std::{process::exit, fs::{read_to_string, self}};

use earpfile::EarpFileWriter;
use error::AssemblerError;
use minicbor::Encoder;
use opcodemap::load_opcode_map;
use options::{parse_config, Config};
use parser::{ParseStatement, load_source_file};
use suite::Suite;
use assemble::{Assemble};

fn debug(config: &Config, str: &str, min: u32) {
    if config.verbose >= min {
        println!("{}",str);
    }
}

fn load_file(path: &str) -> Result<String,AssemblerError> {
    read_to_string(path).map_err(|e|  AssemblerError::FileError(e.to_string()))
}

fn save_file(config: &Config, path: &str, data: &Vec<u8>) -> Result<(),AssemblerError> {
    debug(config,&format!("Writing {}",path),1);
    fs::write(path,data).map_err(|e|  AssemblerError::FileError(e.to_string()))
}

fn load_opcode_map_file(config: &Config, suite: &mut Suite, name: &str, contents: &str) -> Result<(),AssemblerError> {
    debug(config,"Loading default maps",2); 
    let maps = load_opcode_map(contents)
        .map_err(|e| e.add_context(&format!("ERROR: loading {}",name)))?;
    for map in maps {
        debug(config,&format!("Loading instruction set map {}",map.identifier().to_string()),2);
        suite.add_instruction_set(map);
    }
    Ok(())
}

// XXX source root
fn write_earp_file(config: &Config, earp_file: &EarpFileWriter) -> Result<(),AssemblerError> {
    let mut out = vec![];
    let mut encoder = Encoder::new(&mut out);
    encoder.encode(earp_file)
        .map_err(|e| AssemblerError::CannotSerialize(e.to_string()))?;
    save_file(&config,&config.object_file,&out)?;
    Ok(())
}

fn run(config: &Config) -> Result<(),AssemblerError> {
    if config.verbose > 0 {
        println!("Assembling output file {}",config.object_file);
    }
    let mut suite = Suite::new();
    suite.source_loader_mut().add_search_path(".");
    if !config.no_default_maps {
        load_opcode_map_file(config, &mut suite, "default maps", include_str!("maps/standard.map"))?;
    }
    for filename in &config.additional_maps {
        let filedata = load_file(filename)?;
        load_opcode_map_file(config, &mut suite, &format!("'{}'",filename), &filedata)?;
    }
    let mut assembler = Assemble::new(&suite);
    for source_file in &config.source_files {
        assembler.add_source(&load_file(&source_file)?,Some(source_file))?;
    }
    assembler.assemble()?;
    write_earp_file(config,&assembler.into_earpfile())?;
    Ok(())
}

pub fn main() {
    let config = match parse_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\n{}\n",e);
            exit(1);
        }
    };
    match run(&config) {
        Ok(()) => {},
        Err(e) => {
            eprintln!("\n{}\n",e);
            exit(1);
        }        
    }
}
