use crate::testutil::{test_suite, assemble, no_error};
use earp_runtime::earpfile::{earpfilereader::EarpFileReader, toplevel::TopLevel};
use minicbor::{Decoder, Decode};
use peregrine_cli_toolkit::hexdump;

#[test]
fn earpfile_reader_smoke() {
    let suite = test_suite();
    let binary = assemble(&suite,include_str!("test/earpfilereader.earp"));
    let earpfile = no_error(EarpFileReader::new(&binary));
    println!("{}",hexdump(&binary));
}

#[test]
fn test_toplevel_smoke() {
    let suite = test_suite();
    let binary = assemble(&suite,include_str!("test/earpfilereader.earp"));
    let mut decoder = Decoder::new(&binary);
    let top_level = no_error(TopLevel::decode(&mut decoder));
    println!("{:?}",&top_level);  
}