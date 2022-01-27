use crate::{suite::suite::Suite, core::error::AssemblerError, suite::assets::AssetSource, suite::fileloader::FileLoader, load_opcode_map, Assemble, earpfile::command::Command};

pub fn no_error<T,E>(res: Result<T, E>) -> T where E: ToString {
    match res {
        Ok(v) => v,
        Err(e) => { 
            println!("unexpected error: {}",e.to_string());
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

pub(crate) fn test_suite() -> Suite {
    let mut suite = Suite::new();
    let mut file_asset_loader = FileLoader::new();
    file_asset_loader.add_search_path(".");
    suite.add_loader(AssetSource::File,file_asset_loader);
    for set in no_error(load_opcode_map(include_str!("../test/test.map"))) {
        suite.add_instruction_set(set);
    }
    suite
}

pub(crate) fn build<'t>(suite: &'t Suite, contents: &str) -> Result<Vec<Command>,AssemblerError> {
    let mut assembler = Assemble::new(&suite);
    assembler.add_source(contents,None)?;
    assembler.assemble()?;
    Ok(assembler.into_earpfile().commands().to_vec())
}
