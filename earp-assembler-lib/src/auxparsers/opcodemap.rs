use std::{collections::HashMap, fmt::Display};

use pest_consume::{ match_nodes, Parser, Error };
use crate::{suite::instructionset::{InstructionSet, InstructionSetId}, core::error::AssemblerError};

#[derive(Debug,Clone,PartialEq)]
pub(crate) enum ArgType {
    Any,
    Jump,
    Register
}

impl Display for ArgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgType::Any => write!(f,"any"),
            ArgType::Jump => write!(f,"jump"),
            ArgType::Register => write!(f,"register"),
        }
    }
}

#[derive(Debug,Clone,PartialEq)]
pub(crate) enum ArgSpec {
    Any,
    Specific(Vec<Vec<ArgType>>)
}

impl Display for ArgSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgSpec::Any => write!(f,"any"),
            ArgSpec::Specific(s) => {
                let mut groups_out = vec![];
                for group in s {
                    let group_out = group.iter().map(|x| format!("{}",x)).collect::<Vec<_>>();
                    groups_out.push(format!("[{}]",group_out.join(", ")));
                }
                write!(f,"{}",groups_out.join(" or "))
            }
        }
    }
}

#[derive(Parser)]
#[grammar = "auxparsers/opcodemap.pest"]
struct EarpOpcodeMapParser;

#[allow(unused)]
type PestResult<T> = std::result::Result<T, Error<Rule>>;
#[allow(unused)]
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl EarpOpcodeMapParser {
    fn EOI(_input: Node) -> PestResult<()> { Ok(()) }

    fn set(input: Node) -> PestResult<&str> { Ok(input.as_str()) }
    fn name(input: Node) -> PestResult<&str> { Ok(input.as_str()) }

    fn version(input: Node) -> PestResult<u64> {
        input.as_str().parse::<u64>() .map_err(|e| input.error(e))
    }

    fn opcode(input: Node) -> PestResult<u64> {
        input.as_str().parse::<u64>() .map_err(|e| input.error(e))
    }

    fn identifier(input: Node) -> PestResult<InstructionSet> {
        Ok(match_nodes!(input.into_children();
            [set(s),version(v)] => InstructionSet::new(&InstructionSetId(s.to_string(),v))
        ))
    }

    fn identifiers(input: Node) -> PestResult<Vec<InstructionSet>> {
        Ok(match_nodes!(input.into_children();
            [identifier(id)..] => id.collect()
        ))
    }

    fn argany(_input: Node) -> PestResult<()> { Ok(()) }
    fn argnone(_input: Node) -> PestResult<()> { Ok(()) }

    fn argone(input: Node) -> PestResult<ArgType> {
        match input.as_str() {
            "r" => Ok(ArgType::Register),
            "a" => Ok(ArgType::Any),
            "j" => Ok(ArgType::Jump),
            _ => Err(input.error("disallowed argument code"))
        }
    }

    fn argbunch(input: Node) -> PestResult<Option<Vec<ArgType>>> {
        Ok(match_nodes!(input.into_children();
            [argnone(_)] => Some(vec![]),
            [argone(s)..] => Some(s.collect()),
            [argany(_)] => None
        ))
    }

    fn argspec(input: Node) -> PestResult<ArgSpec> {
        Ok(match_nodes!(input.into_children();
            [argbunch(bunches)..] => {
                let mut out = vec![];
                for bunch in bunches {
                    if let Some(argtype) = bunch {
                        out.push(argtype);
                    } else {
                        return Ok(ArgSpec::Any)
                    }
                }
                ArgSpec::Specific(out)
            }
        ))
    }
    
    fn map_line(input: Node) -> PestResult<(u64,&str,ArgSpec)> {
        Ok(match_nodes!(input.into_children();
            [opcode(p),name(n),argspec(s)] => (p,n,s)
        ))
    }

    fn section(input: Node) -> PestResult<Vec<InstructionSet>> {
        let mut out = HashMap::new();
        let node = input.clone();
        match_nodes!(input.into_children();
            [identifiers(mut sets),map_line(lines)..] => { 
                for (opcode,name,argspec) in lines {
                    for set in &mut sets {
                        let set = out.entry(set.identifier().clone()).or_insert_with(|| InstructionSet::new(set.identifier()));
                        set.add(name,opcode,argspec.clone()).map_err(|e| node.error(e.to_string()))?;
                    }
                }
                ()
            }
        );
        Ok(out.drain().map(|(_,v)| v).collect())
    }

    fn document(input: Node) -> PestResult<Vec<InstructionSet>> {
        let mut out = HashMap::new();
        let node = input.clone();
        match_nodes!(input.into_children();
            [section(sections)..,_EOI] => { 
                for section in sections {
                    for in_set in &section {
                        let set = out.entry(in_set.identifier().clone()).or_insert_with(|| InstructionSet::new(in_set.identifier()));
                        set.merge(in_set).map_err(|e| node.error(e))?;
                    }
                }
                ()
            }
        );
        Ok(out.drain().map(|(_,v)| v).collect())
    }
}

fn parse_opcode_map(map: &str) -> PestResult<Vec<InstructionSet>> {
    let input = EarpOpcodeMapParser::parse(Rule::document, map)?.single()?;
    EarpOpcodeMapParser::document(input)
}


pub fn load_opcode_map(map: &str) -> Result<Vec<InstructionSet>,AssemblerError> {
    parse_opcode_map(map).map_err(|e| AssemblerError::BadOpcodeMap(e.to_string()))
}

// effectively tested by tests in instructionset.rs
