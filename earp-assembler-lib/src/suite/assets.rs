use minicbor::{encode::{Write, Error}, Encode, Encoder};
use std::{collections::HashMap, fmt::Display };

use crate::{core::error::AssemblerError, Suite, auxparsers::hexfile::load_hexfile};

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum AssetSource {
    File
}

impl Display for AssetSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match  self {
            AssetSource::File => "file"
        };
        write!(f,"{}",s)
    }
}

#[derive(Clone,Debug,PartialEq)]
pub enum AssetFormat {
    Raw,
    String,
    Hex
}

impl Display for AssetFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match  self {
            AssetFormat::Hex => "hex",
            AssetFormat::String => "string",
            AssetFormat::Raw => "raw"
        };
        write!(f,"{}",s)
    }
}

enum AssetData {
    Bytes(Vec<u8>),
    String(String)
}

impl Encode for AssetData {
    fn encode<W: Write>(&self, encoder: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        match self {
            AssetData::Bytes(b) => { encoder.bytes(b)?; },
            AssetData::String(s) => { encoder.str(s)?; }
        }
        Ok(())
    }
}

pub trait AssetLoad {
    fn load_bytes(&self) -> Result<Vec<u8>,AssemblerError>;
    fn load_string(&self) -> Result<String,AssemblerError>;
}

pub trait AssetLoader {
    fn make_load<'a>(&'a self, path: &str, context_path: &Option<String>, search: bool) -> Result<Box<dyn AssetLoad + 'a>,AssemblerError>;
}


pub(crate) struct Assets<'t> {
    asset: HashMap<String,AssetData>,
    suite: &'t Suite
}

impl<'t> Assets<'t> {
    pub(crate) fn new(suite: &'t Suite) -> Assets<'t> {
        Assets {
            asset: HashMap::new(),
            suite
        }
    }

    fn loader(&self, source: &AssetSource) -> Result<&Box<dyn AssetLoader>,AssemblerError> {
        self.suite.get_loader(source).ok_or_else(|| AssemblerError::FileError(format!("No loader for source: {:?}",source)))
    }

    fn load_string(&self, source: &AssetSource, path: &str, context_path: &Option<String>) -> Result<String,AssemblerError> {
        self.loader(source)?.make_load(path,context_path,true)?.load_string()
    }

    fn load_bytes(&self, source: &AssetSource, path: &str, context_path: &Option<String>) -> Result<Vec<u8>,AssemblerError> {
        self.loader(source)?.make_load(path,context_path,true)?.load_bytes()
    }

    fn load(&self, format: &AssetFormat, source: &AssetSource, path: &str, context_path: &Option<String>) -> Result<AssetData,AssemblerError> {
        Ok(match format {
            AssetFormat::String => AssetData::String(self.load_string(source,path,context_path)?),
            AssetFormat::Raw => AssetData::Bytes(self.load_bytes(source,path,context_path)?),
            AssetFormat::Hex => AssetData::Bytes(load_hexfile(&self.load_string(source,path,context_path)?)?),
        })
    }

    pub(crate) fn add(&mut self, name: &str, format: &AssetFormat, source: &AssetSource, path: &str, context_path: &Option<String>) -> Result<(),AssemblerError> {
        let data = self.load(format,source,path,context_path)?;
        if self.asset.contains_key(name) {
            return Err(AssemblerError::DuplicateAssetName(name.to_string()));
        }
        self.asset.insert(name.to_string(),data);
        Ok(())
    }
}

impl<'t> Encode for Assets<'t> {
    fn encode<W: Write>(&self, encoder: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        let mut ids = self.asset.keys().collect::<Vec<_>>();
        ids.sort();
        encoder.begin_map()?;
        for id in ids {
            encoder.str(id)?.encode(self.asset.get(id).unwrap())?;
        }
        encoder.end()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use minicbor::Encoder;
    use peregrine_cli_toolkit::hexdump;

    use crate::{core::testutil::{no_error, test_suite, yes_error}, suite::assets::{AssetSource, AssetFormat}, assemble::parser::{earp_parse, ParseStatement}, auxparsers::hexfile::load_hexfile, Assemble};

    #[test]
    fn test_asset_parse() {
        assert_eq!(no_error(earp_parse(include_str!("../test/assets/smoke.earp"))),
        vec![
            ParseStatement::InstructionsDecl(None, "std".to_string(), 0), 
            ParseStatement::AssetDecl("test".to_string(), AssetFormat::Raw, AssetSource::File, "raw-asset.bin".to_string()),
            ParseStatement::Program("test".to_string()),
            ParseStatement::Instruction(None, "halt".to_string(), vec![])
        ]);
    }

    #[test]
    fn test_asset_assemble() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("../test/assets/smoke.earp"),Some("src/test/assets/assets.earp")));
        no_error(assembler.assemble());
        let mut out = vec![];
        let mut encoder = Encoder::new(&mut out);
        no_error(encoder.encode(&assembler.into_earpfile()));
        let cmp = no_error(load_hexfile(include_str!("../test/assets/smoke.hex")));
        print!("{}",hexdump(&out));
        assert_eq!(cmp,out);
    }

    #[test]
    fn test_asset_missing() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("../test/assets/missing.earp"),Some("src/test/assets/assets.earp")));
        let file = yes_error(assembler.assemble()).to_string();
        assert!(file.contains("missing-raw-asset.bin"));
        assert!(file.contains("no such path"));
    }

    #[test]
    fn test_asset_duplicate() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("../test/assets/duplicate.earp"),Some("src/test/assets/duplicate.earp")));
        let file = yes_error(assembler.assemble()).to_string();
        println!("{}",file);
        assert!(file.contains("test"));
        assert!(file.to_lowercase().contains("duplicate asset name"));
    }

    #[test]
    fn test_string_asset_assemble() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("../test/assets/smoke-string.earp"),Some("src/test/assets/smoke-string.earp")));
        no_error(assembler.assemble());
        let mut out = vec![];
        let mut encoder = Encoder::new(&mut out);
        no_error(encoder.encode(&assembler.into_earpfile()));
        let cmp = no_error(load_hexfile(include_str!("../test/assets/smoke-string.hex")));
        print!("{}",hexdump(&out));
        assert_eq!(cmp,out);
    }

    #[test]
    fn test_hex_asset_assemble() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("../test/assets/smoke-hex.earp"),Some("src/test/assets/smoke-hex.earp")));
        no_error(assembler.assemble());
        let mut out = vec![];
        let mut encoder = Encoder::new(&mut out);
        no_error(encoder.encode(&assembler.into_earpfile()));
        let cmp = no_error(load_hexfile(include_str!("../test/assets/smoke.hex")));
        print!("{}",hexdump(&out));
        assert_eq!(cmp,out);
    }

    #[test]
    fn test_asset_bad_source() {
        let e = yes_error(earp_parse(include_str!("../test/assets/bad-source.earp"))).to_string();
        assert!(e.contains("asset_source"));
    }

    #[test]
    fn test_asset_bad_format() {
        let e = yes_error(earp_parse(include_str!("../test/assets/bad-format.earp"))).to_string();
        assert!(e.contains("asset_format"));
    }

    #[test]
    fn test_asset_bad_hex_file() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("../test/assets/bad-hexfile.earp"),Some("src/test/assets/bad-hexfile.earp")));
        let e = yes_error(assembler.assemble()).to_string();
        assert!(e.to_lowercase().contains("bad hex file"));
    }

    #[test]
    fn test_asset_bad_string_file() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("../test/assets/bad-string.earp"),Some("src/test/assets/bad-string.earp")));
        let e = yes_error(assembler.assemble()).to_string();
        assert!(e.to_lowercase().contains("stream did not contain valid utf-8"));
    }
}
