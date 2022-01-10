mod assemble;
mod command;
mod earpfile;
mod error;
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

use std::{process::exit, fs::read_to_string};

use error::EarpAssemblerError;
use opcodemap::load_opcode_map;
use options::{parse_config, Config};
use parser::{EarpAssemblyStatement, load_source_file};
use suite::Suite;
use assemble::assemble;

fn debug(config: &Config, str: &str) {
    if config.verbose > 1 {
        println!("{}",str);
    }
}

fn load_file(path: &str) -> Result<String,EarpAssemblerError> {
    read_to_string(path).map_err(|e|  EarpAssemblerError::FileError(e.to_string()))
}

fn load_opcode_map_file(config: &Config, suite: &mut Suite, name: &str, contents: &str) -> Result<(),EarpAssemblerError> {
    debug(config,"Loading default maps"); 
    let maps = load_opcode_map(contents)
        .map_err(|e| e.add_context(&format!("ERROR: loading {}",name)))?;
    for map in maps {
        debug(config,&format!("Loading instruction set map {}",map.identifier().to_string()));
        suite.add(map);
    }
    Ok(())
}

fn load_source(path: &str) -> Result<Vec<EarpAssemblyStatement>,EarpAssemblerError> {
    let filedata = load_file(path)?;
    load_source_file(&filedata)
}

fn load_sources(config: &Config) -> Result<Vec<EarpAssemblyStatement>,EarpAssemblerError> {
    let mut out = vec![];
    for path in &config.source_files {
        out.append(&mut load_source(path)?);
    }
    Ok(out)
}

fn run(config: &Config) -> Result<(),EarpAssemblerError> {
    if config.verbose > 0 {
        println!("Assembling output file {}",config.object_file);
    }
    let mut suite = Suite::new();
    if !config.no_default_maps {   
        load_opcode_map_file(config, &mut suite, "default maps", include_str!("maps/standard.map"))?;
    }
    for filename in &config.additional_maps {
        let filedata = load_file(filename)?;
        load_opcode_map_file(config, &mut suite, &format!("'{}'",filename), &filedata)?;
    }
    let source = load_sources(config)?;
    let earp_file = assemble(&suite,&source)?;
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
