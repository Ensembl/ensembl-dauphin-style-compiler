use std::fmt::Debug;

use earp_assembler_lib::{ FileLoader, AssetSource, load_opcode_map, Assemble };
use earp_assembler_lib::Suite as AssemblerSuite;
use minicbor::Encoder;

pub fn no_error<T,E>(res: Result<T, E>) -> T where E: Debug {
    match res {
        Ok(v) => v,
        Err(e) => { 
            println!("unexpected error: {:?}",e);
            assert!(false);
            panic!();
        }
    }
}

pub fn yes_error<T,E>(res: Result<T, E>) -> E {
    match res {
        Ok(_) => {
            println!("expected error, didn't get one!");
            assert!(false);
            panic!();
        }
        Err(e) => e
    }
}

pub(crate) fn test_assembler_suite() -> AssemblerSuite {
    let mut suite = AssemblerSuite::new();
    let mut file_asset_loader = FileLoader::new();
    file_asset_loader.add_search_path(".");
    suite.add_loader(AssetSource::File,file_asset_loader);
    for set in no_error(load_opcode_map(include_str!("test/test.map"))) {
        suite.add_instruction_set(set);
    }
    suite
}

pub(crate) fn assemble(suite: &AssemblerSuite, source: &str) -> Vec<u8> {
    let mut assembler = Assemble::new(&suite);
    no_error(assembler.add_source(source,None));
    no_error(assembler.assemble());
    let mut out = vec![];
    let mut encoder = Encoder::new(&mut out);
    no_error(encoder.encode(&assembler.into_earpfile()));
    out
}