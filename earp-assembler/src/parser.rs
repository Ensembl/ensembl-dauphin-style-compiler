use pest_consume::{Error, Parser, match_nodes};

use crate::error::EarpAssemblerError;

#[derive(Clone,Debug,PartialEq)]
pub(crate) enum EarpAssemblyLocation {
    Here(i64),
    Label(String),
    RelativeLabel(String,bool)
}

#[derive(Clone,Debug,PartialEq)]
pub(crate) enum EarpAssemblyOperand {
    Register(usize),
    UpRegister(usize),
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Location(EarpAssemblyLocation)
}

#[derive(Debug,PartialEq)]
pub(crate) enum EarpAssemblyStatement {
    Program(String),
    InstructionsDecl(Option<String>,String,u64),
    Instruction(Option<String>,String,Vec<EarpAssemblyOperand>),
    Label(String),
    RelativeLabel(String),
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

    fn instructions_declaration(input: Node) -> PestResult<EarpAssemblyStatement> {
        Ok(match_nodes!(input.into_children();
            [identifier(set),integer(version)] =>
                EarpAssemblyStatement::InstructionsDecl(None,set.to_string(),version),
            [identifier(prefix),identifier(set),integer(version)] => 
                EarpAssemblyStatement::InstructionsDecl(Some(prefix.to_string()),set.to_string(),version)
        ))
    }

    fn declaration(input: Node) -> PestResult<EarpAssemblyStatement> {
        Ok(match_nodes!(input.into_children(); [instructions_declaration(d)] => d ))
    }

    fn declaration_section(input: Node) -> PestResult<Vec<EarpAssemblyStatement>> {
        Ok(match_nodes!(input.into_children(); [declaration(d)..] => d.collect() ))
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

    fn operand(input: Node) -> PestResult<EarpAssemblyOperand> {
        Ok(match_nodes!(input.into_children();
            [register(r)] => EarpAssemblyOperand::Register(r as usize),
            [upregister(r)] => EarpAssemblyOperand::UpRegister(r as usize),
            [boolean(b)] => EarpAssemblyOperand::Boolean(b),
            [float_tagged(f)] => EarpAssemblyOperand::Float(f),
            [integer(n)] => EarpAssemblyOperand::Integer(n as i64),
            [string(s)] => EarpAssemblyOperand::String(s),
            [rellabelr(b)] => EarpAssemblyOperand::Location(EarpAssemblyLocation::RelativeLabel(b.to_string(),false)),
            [rellabelf(b)] => EarpAssemblyOperand::Location(EarpAssemblyLocation::RelativeLabel(b.to_string(),true)),
            [labelref(b)] => EarpAssemblyOperand::Location(EarpAssemblyLocation::Label(b.to_string())),
            [relative(r)] => EarpAssemblyOperand::Location(EarpAssemblyLocation::Here(r))
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

    fn instruction(input: Node) -> PestResult<EarpAssemblyStatement> {
        Ok(match_nodes!(input.into_children();
            [opcode((prefix,instr)),operand(operands)..] => 
                EarpAssemblyStatement::Instruction(prefix.map(|x| x.to_string()),instr.to_string(),operands.collect())
        ))
    }

    /* LABELS */

    fn program_label(input: Node) -> PestResult<&str> { Ok(match_nodes!(input.into_children(); [identifier(id)] => id )) } 
    fn label(input: Node) -> PestResult<&str> { Ok(match_nodes!(input.into_children(); [identifier(id)] => id )) } 
    fn rellabel(input: Node) -> PestResult<String> { Ok(match_nodes!(input.into_children(); [integer(v)] => v.to_string() )) } 

    /* PROGRAM LINES */

    fn program_line(input: Node) -> PestResult<EarpAssemblyStatement> {
        Ok(match_nodes!(input.into_children();
            [instruction(instr)] => instr,
            [program_label(prog)] => EarpAssemblyStatement::Program(prog.to_string()),
            [label(b)] => EarpAssemblyStatement::Label(b.to_string()),
            [rellabel(b)] => EarpAssemblyStatement::RelativeLabel(b)
        ))
    }

    fn program_section(input: Node) -> PestResult<Vec<EarpAssemblyStatement>> {
        Ok(match_nodes!(input.into_children(); [program_line(d)..] => d.collect() ))
    }

    /*
     * DOCUMENT
     */

    fn document(input: Node) -> PestResult<Vec<EarpAssemblyStatement>> {
        Ok(match_nodes!(input.into_children();
            [declaration_section(mut d),program_section(mut p),_EOI] => { d.append(&mut p); d }
        ))
    }
}

pub(crate) fn earp_parse(contents: &str) -> PestResult<Vec<EarpAssemblyStatement>> {
    let input = AssemblerParser::parse(Rule::document, contents)?.single()?;
    AssemblerParser::document(input)
}

pub(crate) fn load_source_file(source: &str) -> Result<Vec<EarpAssemblyStatement>,EarpAssemblerError> {
    earp_parse(source).map_err(|e| EarpAssemblerError::SyntaxError(e.to_string()))
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::{testutil::{no_error}, parser::{EarpAssemblyStatement, EarpAssemblyOperand, EarpAssemblyLocation}};

    use super::earp_parse;

    #[test]
    fn parse_smoke() {
        assert_eq!(no_error(earp_parse(include_str!("test/test.earp"))),
            vec![
                EarpAssemblyStatement::InstructionsDecl(None,"std".to_string(),0),
                EarpAssemblyStatement::InstructionsDecl(Some("c".to_string()),"console".to_string(),0),

                EarpAssemblyStatement::Program("test1".to_string()),
                EarpAssemblyStatement::Label("label".to_string()),
                EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                    EarpAssemblyOperand::Register(0),
                    EarpAssemblyOperand::String("hello, \"world\"".to_string())]),
                EarpAssemblyStatement::Instruction(Some("c".to_string()), "info".to_string(), vec![
                    EarpAssemblyOperand::Register(0)]),
                EarpAssemblyStatement::Instruction(None, "goto".to_string(), vec![
                    EarpAssemblyOperand::Location(EarpAssemblyLocation::Label("label".to_string()))]),

                EarpAssemblyStatement::Program("test2".to_string()),
                EarpAssemblyStatement::RelativeLabel("1".to_string()),
                EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                    EarpAssemblyOperand::Register(0),
                    EarpAssemblyOperand::String("hello, \"world\"\n".to_string())]),
                EarpAssemblyStatement::Instruction(None, "push".to_string(), vec![]),
                EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                    EarpAssemblyOperand::Register(1),
                    EarpAssemblyOperand::UpRegister(0)]),
                EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                    EarpAssemblyOperand::Register(0),
                    EarpAssemblyOperand::Location(EarpAssemblyLocation::Here(2))]),    
                EarpAssemblyStatement::Instruction(None, "goto".to_string(), vec![
                    EarpAssemblyOperand::Location(EarpAssemblyLocation::Label("printer".to_string()))]),    
                EarpAssemblyStatement::Instruction(None, "pop".to_string(), vec![]),
                EarpAssemblyStatement::Instruction(None, "goto".to_string(), vec![
                    EarpAssemblyOperand::Location(EarpAssemblyLocation::RelativeLabel("1".to_string(),false))]),

                EarpAssemblyStatement::Label("printer".to_string()),
                EarpAssemblyStatement::Instruction(Some("c".to_string()), "info".to_string(), vec![
                    EarpAssemblyOperand::Register(1)]),
                EarpAssemblyStatement::Instruction(Some("c".to_string()), "warn".to_string(), vec![
                    EarpAssemblyOperand::Register(1)]),
                EarpAssemblyStatement::Instruction(None, "goto".to_string(), vec![
                    EarpAssemblyOperand::Register(0)]),    
        ]);
    }

    fn operands(stmts: Vec<EarpAssemblyStatement>) -> Vec<EarpAssemblyOperand> {
        let mut values = vec![];
        for stmt in &stmts {
            match stmt {
                EarpAssemblyStatement::Instruction(prefix,name,args) => {
                    assert_eq!(&None,prefix);
                    assert_eq!("copy",name);
                    assert_eq!(2,args.len());
                    assert_eq!(EarpAssemblyOperand::Register(0),args[0]);
                    values.push(args[1].clone());
                },
                _ => {}
            }
        }
        values
    }

    #[test]
    fn test_parse_floats() {
        let p = no_error(earp_parse(include_str!("test/test-floats.earp")));
        let values = operands(p);

        let mut cmps = vec![
            0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,
            5.,5.,-5.,  5.5,5.5,-5.5,  550000., 550000.,-550000.,
            550000., 550000.,-550000.,  0.000055,  0.000055,  -0.000055
            ];
        cmps.reverse();
        for arg in &values {
            let got = match arg {
                EarpAssemblyOperand::Float(x) => x,
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
        let bads = include_str!("test/bad.earp");
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
        no_error(earp_parse(include_str!("test/nodecls.earp")));
    }

    #[test]
    fn test_no_prog() {
        no_error(earp_parse(include_str!("test/noprog.earp")));
    }

    #[test]
    fn test_empty() {
        no_error(earp_parse(include_str!("test/empty.earp")));
    }

    #[test]
    fn test_parse_strings() {
        let p = no_error(earp_parse(include_str!("test/test-strings.earp")));
        let values = operands(p);

        let mut cmps = vec![
            "hello", "hello\\", 
            "hello\0007","hello\0010","hello\0014","hello\n","hello\r","hello\t","hello\0013",
            "x\u{0001}x","\u{20AC}z"
        ];
        cmps.reverse();
        for arg in &values {
            let got = match arg {
                EarpAssemblyOperand::String(x) => x,
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
        assert_eq!(no_error(earp_parse(include_str!("test/test-misc.earp"))),
        vec![
            EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                EarpAssemblyOperand::Register(0),
                EarpAssemblyOperand::Boolean(true)]),
            EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                EarpAssemblyOperand::Register(0),
                EarpAssemblyOperand::Boolean(false)]),
            EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                EarpAssemblyOperand::Register(0),
                EarpAssemblyOperand::Location(EarpAssemblyLocation::Here(0))]),    
            EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                EarpAssemblyOperand::Register(0),
                EarpAssemblyOperand::Location(EarpAssemblyLocation::Here(31))]),    
            EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                EarpAssemblyOperand::Register(0),
                EarpAssemblyOperand::Location(EarpAssemblyLocation::Here(-42))]),                
            EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                EarpAssemblyOperand::Register(0),
                EarpAssemblyOperand::Location(EarpAssemblyLocation::Label("test".to_string()))]),
            EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                EarpAssemblyOperand::Register(0),
                EarpAssemblyOperand::Location(EarpAssemblyLocation::RelativeLabel("2".to_string(),false))]),
            EarpAssemblyStatement::Instruction(None, "copy".to_string(), vec![
                EarpAssemblyOperand::Register(0),
                EarpAssemblyOperand::Location(EarpAssemblyLocation::RelativeLabel("32".to_string(),true))]),
            EarpAssemblyStatement::Program("test2".to_string()),
            EarpAssemblyStatement::Label("program".to_string()),
            EarpAssemblyStatement::RelativeLabel("1".to_string()),
            EarpAssemblyStatement::RelativeLabel("23".to_string()),
            ]);
    }
}
