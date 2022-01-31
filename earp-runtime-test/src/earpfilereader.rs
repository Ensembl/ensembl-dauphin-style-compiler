use crate::testutil::{test_assembler_suite, assemble, no_error};
use earp_runtime::{earpfile::{earpfilereader::EarpFileReader, toplevel::TopLevel}, suite::suite::Suite};
use minicbor::{Decoder, Decode};
use peregrine_cli_toolkit::hexdump;

#[test]
fn earpfile_reader_smoke() {
    let assembler_suite = test_assembler_suite();
    let binary = assemble(&assembler_suite,include_str!("test/earpfilereader.earp"));
    let runtime_suite = Suite::new();
    let earpfile = no_error(EarpFileReader::new(&runtime_suite,&binary));
    println!("{}",hexdump(&binary));
}

#[test]
fn test_toplevel_smoke() {
    let suite = test_assembler_suite();
    let binary = assemble(&suite,include_str!("test/earpfilereader.earp"));
    let mut decoder = Decoder::new(&binary);
    let top_level = no_error(TopLevel::decode(&mut decoder));
    println!("{:?}",&top_level);  
}