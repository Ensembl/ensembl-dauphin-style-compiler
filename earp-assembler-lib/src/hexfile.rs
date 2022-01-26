use pest_consume::{ match_nodes, Parser, Error };

use crate::error::AssemblerError;

enum Unit {
    Offset(u64),
    Data(u8)
}

#[derive(Parser)]
#[grammar = "hexfile.pest"]
struct HexFileParser;

#[allow(unused)]
type PestResult<T> = std::result::Result<T, Error<Rule>>;
#[allow(unused)]
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl HexFileParser {
    fn EOI(_input: Node) -> PestResult<()> { Ok(()) }

    fn hex8(input: Node) -> PestResult<u8> {
        u8::from_str_radix(input.as_str(),16).map_err(|e| input.error(e))
    }

    fn hex64(input: Node) -> PestResult<u64> {
        u64::from_str_radix(input.as_str(),16).map_err(|e| input.error(e))
    }

    fn offset(input: Node) -> PestResult<u64> {
        Ok(match_nodes!(input.into_children();
            [hex64(hex)] => hex
        ))
    }

    fn unit(input: Node) -> PestResult<Unit> {
        Ok(match_nodes!(input.into_children();
            [offset(offset)] => Unit::Offset(offset),
            [hex8(hex)] => Unit::Data(hex)
        ))
    }

    fn contents(input: Node) -> PestResult<Vec<Unit>> {
        let mut out = vec![];
        match_nodes!(input.into_children();
            [unit(unit)..,_EOI] => { 
                out.append(&mut unit.collect());
                ()
            }
        );
        Ok(out)
    }
}

fn parse_hexfile(map: &str) -> PestResult<Vec<Unit>> {
    let input = HexFileParser::parse(Rule::contents, map)?.single()?;
    HexFileParser::contents(input)
}

pub(crate) fn load_hexfile(map: &str) -> Result<Vec<u8>,AssemblerError> {
    let units = parse_hexfile(map).map_err(|e|
        AssemblerError::BadHexFile(format!("hexfile parse error: {}",e))
    )?;
    let mut out = vec![];
    for unit in units.iter() {
        match unit {
            Unit::Offset(offset) => {
                if *offset != out.len() as u64 {
                    return Err(AssemblerError::BadHexFile(format!("encountered @{:04x} at 0x{:04x}",offset,out.len())));
                }
            },
            Unit::Data(d) => {
                out.push(*d);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod test {
    use super::load_hexfile;
    use crate::testutil::{ no_error, yes_error };

    #[test]
    fn hexfile_smoke() {
        let data = include_str!("test/hexfile/smoke.hex");
        let hex = no_error(load_hexfile(data));
        assert_eq!(vec![0,1,2,3,4],hex);
    }

    #[test]
    fn hexfile_bad_offset() {
        let data = include_str!("test/hexfile/bad-offset.hex");
        assert!(yes_error(load_hexfile(data)).to_string().contains("encountered @0005 at 0x0004"));
    }

    #[test]
    fn hexfile_empty() {
        let data = include_str!("test/hexfile/empty.hex");
        let hex = no_error(load_hexfile(data));
        assert_eq!(Vec::<u8>::new(),hex);
    }
}
