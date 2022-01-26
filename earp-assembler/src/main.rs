mod options;

use minicbor::Encoder;
use std::{ fs::{ self, read_to_string }, process::exit };

use earp_assembler_lib::{AssemblerError, Suite, EarpFileWriter, Assemble, AssetSource, FileLoader, load_opcode_map};
use options::{parse_config, Config};

fn debug(config: &Config, str: &str, min: u32) {
    if config.verbose >= min {
        println!("{}",str);
    }
}

fn prepare_suite(config: &Config) -> Suite {
    let mut suite = Suite::new();
    suite.source_loader_mut().add_search_path(".");
    for path in &config.source_paths {
        suite.source_loader_mut().add_search_path(path);
    }
    let mut file_asset_loader = FileLoader::new();
    file_asset_loader.add_search_path(".");
    for path in &config.asset_paths {
        file_asset_loader.add_search_path(path);
    }
    suite.add_loader(AssetSource::File,file_asset_loader);
    suite
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
    let mut suite = prepare_suite(config);
    if !config.no_default_maps {
        load_opcode_map_file(config, &mut suite, "default maps", include_str!("maps/standard.map"))?;
    }
    for filename in &config.additional_maps {
        let filedata = load_file(filename)?;
        load_opcode_map_file(config, &mut suite, &format!("'{}'",filename), &filedata)?;
    }
    let mut assembler = Assemble::new(&suite);
    for source_file in &config.source_files {
        assembler.add_file(source_file)?;
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
