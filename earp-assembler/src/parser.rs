use pest_consume::{Error, Parser, match_nodes};

use crate::{error::AssemblerError, assets::{AssetSource, AssetFormat}};

#[derive(Clone,Debug,PartialEq)]
pub(crate) enum AssemblyLocation {
    Here(i64),
    Label(String),
    RelativeLabel(String,bool)
}

#[derive(Clone,Debug,PartialEq)]
pub(crate) enum ParseOperand {
    Register(usize),
    UpRegister(usize),
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Location(AssemblyLocation)
}

#[derive(Debug,PartialEq)]
pub(crate) enum ParseStatement {
    InstructionsDecl(Option<String>,String,u64),
    AssetDecl(String,AssetFormat,AssetSource,String),
    /**/
    Program(String),
    Instruction(Option<String>,String,Vec<ParseOperand>),
    Label(String),
    RelativeLabel(String),
    Noop
}

impl ParseStatement {
    fn is_noop(&self) -> bool {
        match self {
            ParseStatement::Noop => true,
            _ => false
        }
    }
}

#[derive(Parser)]
#[grammar = "parser.pest"]
struct AssemblerParser;

#[allow(unused)]
type PestResult<T> = std::result::Result<T, Error<Rule>>;
#[allow(unused)]
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl AssemblerParser {
    /*
     * GENERAL
     */

    fn EOI(_input: Node) -> PestResult<()> { Ok(()) }

    fn identifier(input: Node) -> PestResult<&str> { Ok(input.as_str()) }

    fn integer(input: Node) -> PestResult<u64> {
        input.as_str().parse::<u64>() .map_err(|e| input.error(e))
    }

    /*
     * DECLARATIONS
     */

    fn instructions_declaration(input: Node) -> PestResult<ParseStatement> {
        Ok(match_nodes!(input.into_children();
            [identifier(set),integer(version)] =>
                ParseStatement::InstructionsDecl(None,set.to_string(),version),
            [identifier(prefix),identifier(set),integer(version)] => 
                ParseStatement::InstructionsDecl(Some(prefix.to_string()),set.to_string(),version)
        ))
    }

    fn asset_format(input: Node) -> PestResult<AssetFormat> {
        match input.as_str() {
            "raw" => Ok(AssetFormat::Raw),
            "hex" => Ok(AssetFormat::Hex),
            "string" => Ok(AssetFormat::String),
            _ => Err(input.error("disallowed asset format (parser bug"))
        }
    }

    fn asset_source(input: Node) -> PestResult<AssetSource> {
        match input.as_str() {
            "file" => Ok(AssetSource::File),
            _ => Err(input.error("disallowed asset source (parser bug"))
        }
    }

    fn asset_path(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }

    //identifier ~ asset_source ~ asset_source ~ asset_path
    fn asset_declaration(input: Node) -> PestResult<ParseStatement> {
        Ok(match_nodes!(input.into_children();
            [identifier(prefix),asset_format(format),asset_source(source),asset_path(path)] => 
            ParseStatement::AssetDecl(prefix.to_string(),format,source,path)
        ))
    }

    fn declaration(input: Node) -> PestResult<ParseStatement> {
        Ok(match_nodes!(input.into_children();
            [instructions_declaration(d)] => d,
            [asset_declaration(d)] => d,
            [] => ParseStatement::Noop
        ))
    }

    fn declaration_section(input: Node) -> PestResult<Vec<ParseStatement>> {
        Ok(match_nodes!(input.into_children(); [declaration(d)..] => {
            d.filter(|x| !x.is_noop()).collect()
        }))
    }

    /*
     * OPERANDS
     */

    /* FLOATS */

    fn float(input: Node) -> PestResult<f64> {
        input.as_str().parse::<f64>().map_err(|e| input.error(e))
    }

    fn float_tagged(input: Node) -> PestResult<f64> { Ok(match_nodes!(input.into_children(); [float(v)] => v )) }

    /* STRINGS */

    fn plain_char(input: Node) -> PestResult<&str> { Ok(input.as_str()) }

    fn escaped_char(input: Node) -> PestResult<&str> { 
        Ok(match input.as_str() {
            "0" => "\0",
            "a" => "\0007",
            "b" => "\0010",
            "f" => "\0014",
            "n" => "\n",
            "r" => "\r",
            "t" => "\t",
            "v" => "\0013",
            x => x
        })
    }

    fn hex_escape(input: Node) -> PestResult<String> {
        let v = u32::from_str_radix(input.as_str(),16).map_err(|e| input.error(e))?;
        let c = char::from_u32(v).ok_or_else(|| input.error("bad unicode character"))?;
        Ok(c.to_string())
    }

    fn character(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
            [plain_char(c)] => c.to_string(),
            [escaped_char(c)] => c.to_string(),
            [hex_escape(c)] => c
        ))
    }

    fn string(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
            [character(c)..] => c.collect::<Vec<_>>().join("")
        ))
    }

    /* BOOLEAN */

    fn boolean(input: Node) -> PestResult<bool> { 
        Ok(match input.as_str() {
            "false" => false,
            "true" => true,
            x => panic!("unexpected boolean constant: '{}'",x)
        })
    }

    /* RELATIVE OFFSETS */

    fn relplus(input: Node) -> PestResult<u64> { Ok(match_nodes!(input.into_children(); [integer(v)] => v )) } 
    fn relminus(input: Node) -> PestResult<u64> { Ok(match_nodes!(input.into_children(); [integer(v)] => v )) } 

    fn relative(input: Node) -> PestResult<i64> {
        Ok(match_nodes!(input.into_children();
            [] => 0,
            [relplus(v)] => v as i64,
            [relminus(v)] => -(v as i64)
        ))
    }

    /* REGISTERS */

    fn register(input: Node) -> PestResult<u64> { Ok(match_nodes!(input.into_children(); [integer(v)] => v )) } 
    fn upregister(input: Node) -> PestResult<u64> { Ok(match_nodes!(input.into_children(); [integer(v)] => v )) } 

    /* LABEL REFERENCES */

    fn labelref(input: Node) -> PestResult<&str> {
        Ok(match_nodes!(input.into_children();
            [identifier(id)] => id
        ))
    }

    fn rellabelf(input: Node) -> PestResult<u64> { Ok(match_nodes!(input.into_children(); [integer(v)] => v )) } 
    fn rellabelr(input: Node) -> PestResult<u64> { Ok(match_nodes!(input.into_children(); [integer(v)] => v )) } 

    /* OPERANDS */

    fn operand(input: Node) -> PestResult<ParseOperand> {
        Ok(match_nodes!(input.into_children();
            [register(r)] => ParseOperand::Register(r as usize),
            [upregister(r)] => ParseOperand::UpRegister(r as usize),
            [boolean(b)] => ParseOperand::Boolean(b),
            [float_tagged(f)] => ParseOperand::Float(f),
            [integer(n)] => ParseOperand::Integer(n as i64),
            [string(s)] => ParseOperand::String(s),
            [rellabelr(b)] => ParseOperand::Location(AssemblyLocation::RelativeLabel(b.to_string(),false)),
            [rellabelf(b)] => ParseOperand::Location(AssemblyLocation::RelativeLabel(b.to_string(),true)),
            [labelref(b)] => ParseOperand::Location(AssemblyLocation::Label(b.to_string())),
            [relative(r)] => ParseOperand::Location(AssemblyLocation::Here(r))
        ))
    }

    /*
     * PROGRAM LINES
     */

    fn opcode(input: Node) -> PestResult<(Option<&str>,&str)> {
        Ok(match_nodes!(input.into_children();
            [identifier(prefix),identifier(id)] => (Some(prefix),id),
            [identifier(id)] => (None,id)
        ))
    }

    /* INSTRUCTIONS */

    fn instruction(input: Node) -> PestResult<ParseStatement> {
        Ok(match_nodes!(input.into_children();
            [opcode((prefix,instr)),operand(operands)..] => 
                ParseStatement::Instruction(prefix.map(|x| x.to_string()),instr.to_string(),operands.collect())
        ))
    }

    /* LABELS */

    fn program_label(input: Node) -> PestResult<&str> { Ok(match_nodes!(input.into_children(); [identifier(id)] => id )) } 
    fn label(input: Node) -> PestResult<&str> { Ok(match_nodes!(input.into_children(); [identifier(id)] => id )) } 
    fn rellabel(input: Node) -> PestResult<String> { Ok(match_nodes!(input.into_children(); [integer(v)] => v.to_string() )) } 

    /* PROGRAM LINES */

    fn program_line(input: Node) -> PestResult<ParseStatement> {
        Ok(match_nodes!(input.into_children();
            [instruction(instr)] => instr,
            [program_label(prog)] => ParseStatement::Program(prog.to_string()),
            [label(b)] => ParseStatement::Label(b.to_string()),
            [rellabel(b)] => ParseStatement::RelativeLabel(b),
            [] => ParseStatement::Noop
        ))
    }

    fn program_section(input: Node) -> PestResult<Vec<ParseStatement>> {
        Ok(match_nodes!(input.into_children(); [program_line(d)..] => { 
            d.filter(|x| !x.is_noop()).collect() 
        } ))
    }

    /*
     * DOCUMENT
     */

    fn document(input: Node) -> PestResult<Vec<ParseStatement>> {
        Ok(match_nodes!(input.into_children();
            [declaration_section(mut d),program_section(mut p),_EOI] => { d.append(&mut p); d }
        ))
    }
}

pub(crate) fn earp_parse(contents: &str) -> PestResult<Vec<ParseStatement>> {
    let input = AssemblerParser::parse(Rule::document, contents)?.single()?;
    AssemblerParser::document(input)
}

pub(crate) fn load_source_file(source: &str) -> Result<Vec<ParseStatement>,AssemblerError> {
    earp_parse(source).map_err(|e| AssemblerError::SyntaxError(e.to_string()))
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::{testutil::{no_error}, parser::{ParseStatement, ParseOperand, AssemblyLocation}};

    use super::earp_parse;

    #[test]
    fn parse_smoke() {
        assert_eq!(no_error(earp_parse(include_str!("test/test.earp"))),
            vec![
                ParseStatement::InstructionsDecl(None,"std".to_string(),0),
                ParseStatement::InstructionsDecl(Some("c".to_string()),"console".to_string(),0),

                ParseStatement::Program("test1".to_string()),
                ParseStatement::Label("label".to_string()),
                ParseStatement::Instruction(None, "copy".to_string(), vec![
                    ParseOperand::Register(0),
                    ParseOperand::String("hello, \"world\"".to_string())]),
                ParseStatement::Instruction(Some("c".to_string()), "info".to_string(), vec![
                    ParseOperand::Register(0)]),
                ParseStatement::Instruction(None, "goto".to_string(), vec![
                    ParseOperand::Location(AssemblyLocation::Label("label".to_string()))]),

                ParseStatement::Program("test2".to_string()),
                ParseStatement::RelativeLabel("1".to_string()),
                ParseStatement::Instruction(None, "copy".to_string(), vec![
                    ParseOperand::Register(0),
                    ParseOperand::String("hello, \"world\"\n".to_string())]),
                ParseStatement::Instruction(None, "push".to_string(), vec![]),
                ParseStatement::Instruction(None, "copy".to_string(), vec![
                    ParseOperand::Register(1),
                    ParseOperand::UpRegister(0)]),
                ParseStatement::Instruction(None, "copy".to_string(), vec![
                    ParseOperand::Register(0),
                    ParseOperand::Location(AssemblyLocation::Here(2))]),    
                ParseStatement::Instruction(None, "goto".to_string(), vec![
                    ParseOperand::Location(AssemblyLocation::Label("printer".to_string()))]),    
                ParseStatement::Instruction(None, "pop".to_string(), vec![]),
                ParseStatement::Instruction(None, "goto".to_string(), vec![
                    ParseOperand::Location(AssemblyLocation::RelativeLabel("1".to_string(),false))]),

                ParseStatement::Label("printer".to_string()),
                ParseStatement::Instruction(Some("c".to_string()), "info".to_string(), vec![
                    ParseOperand::Register(1)]),
                ParseStatement::Instruction(Some("c".to_string()), "warn".to_string(), vec![
                    ParseOperand::Register(1)]),
                ParseStatement::Instruction(None, "goto".to_string(), vec![
                    ParseOperand::Register(0)]),    
        ]);
    }

    fn operands(stmts: Vec<ParseStatement>) -> Vec<ParseOperand> {
        let mut values = vec![];
        for stmt in &stmts {
            match stmt {
                ParseStatement::Instruction(prefix,name,args) => {
                    assert_eq!(&None,prefix);
                    assert_eq!("copy",name);
                    assert_eq!(2,args.len());
                    assert_eq!(ParseOperand::Register(0),args[0]);
                    values.push(args[1].clone());
                },
                _ => {}
            }
        }
        values
    }

    #[test]
    fn test_parse_floats() {
        let p = no_error(earp_parse(include_str!("test/parser/floats.earp")));
        let values = operands(p);

        let mut cmps = vec![
            0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,
            5.,5.,-5.,  5.5,5.5,-5.5,  550000., 550000.,-550000.,
            550000., 550000.,-550000.,  0.000055,  0.000055,  -0.000055
            ];
        cmps.reverse();
        for arg in &values {
            let got = match arg {
                ParseOperand::Float(x) => x,
                _ => { assert!(false); panic!(); }
            };
            let cmp = cmps.pop().unwrap();
            if cmp.partial_cmp(got) != Some(Ordering::Equal) {
                println!("{} vs {}\n",got,cmp);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_bad() {
        let bads = include_str!("test/parser/bad.earp");
        for bad in bads.lines() {
            if !bad.is_empty() {
                if let Ok(p) = earp_parse(bad) {
                    println!("bad success '{}': {:?}",bad,p);
                }
            }
        }
    }

    #[test]
    fn test_no_decls() {
        no_error(earp_parse(include_str!("test/parser/nodecls.earp")));
    }

    #[test]
    fn test_no_prog() {
        no_error(earp_parse(include_str!("test/parser/noprog.earp")));
    }

    #[test]
    fn test_empty() {
        no_error(earp_parse(include_str!("test/parser/empty.earp")));
    }

    #[test]
    fn test_parse_strings() {
        let p = no_error(earp_parse(include_str!("test/parser/strings.earp")));
        let values = operands(p);

        let mut cmps = vec![
            "hello", "hello\\", 
            "hello\0007","hello\0010","hello\0014","hello\n","hello\r","hello\t","hello\0013",
            "x\u{0001}x","\u{20AC}z"
        ];
        cmps.reverse();
        for arg in &values {
            let got = match arg {
                ParseOperand::String(x) => x,
                _ => { assert!(false); panic!(); }
            };
            let cmp = cmps.pop().unwrap();
            if cmp != got {
                println!("{:?} vs {:?}\n",got,cmp);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_parse_misc() {
        assert_eq!(no_error(earp_parse(include_str!("test/parser/misc.earp"))),
        vec![
            ParseStatement::Instruction(None, "copy".to_string(), vec![
                ParseOperand::Register(0),
                ParseOperand::Boolean(true)]),
            ParseStatement::Instruction(None, "copy".to_string(), vec![
                ParseOperand::Register(0),
                ParseOperand::Boolean(false)]),
            ParseStatement::Instruction(None, "copy".to_string(), vec![
                ParseOperand::Register(0),
                ParseOperand::Location(AssemblyLocation::Here(0))]),    
            ParseStatement::Instruction(None, "copy".to_string(), vec![
                ParseOperand::Register(0),
                ParseOperand::Location(AssemblyLocation::Here(31))]),    
            ParseStatement::Instruction(None, "copy".to_string(), vec![
                ParseOperand::Register(0),
                ParseOperand::Location(AssemblyLocation::Here(-42))]),                
            ParseStatement::Instruction(None, "copy".to_string(), vec![
                ParseOperand::Register(0),
                ParseOperand::Location(AssemblyLocation::Label("test".to_string()))]),
            ParseStatement::Instruction(None, "copy".to_string(), vec![
                ParseOperand::Register(0),
                ParseOperand::Location(AssemblyLocation::RelativeLabel("2".to_string(),false))]),
            ParseStatement::Instruction(None, "copy".to_string(), vec![
                ParseOperand::Register(0),
                ParseOperand::Location(AssemblyLocation::RelativeLabel("32".to_string(),true))]),
            ParseStatement::Program("test2".to_string()),
            ParseStatement::Label("program".to_string()),
            ParseStatement::RelativeLabel("1".to_string()),
            ParseStatement::RelativeLabel("23".to_string()),
        ]);
    }

    #[test]
    fn test_no_eol() {
        assert_eq!(no_error(earp_parse(include_str!("test/parser/no-eol.earp"))),
            vec![
                ParseStatement::Instruction(None,"halt".to_string(),vec![])
            ]);
    }
    
    #[test]
    fn test_no_eol2() {
        assert_eq!(no_error(earp_parse(include_str!("test/parser/no-eol2.earp"))),
            vec![
                ParseStatement::InstructionsDecl(None,"std".to_string(),0)
            ]);
    }
}
