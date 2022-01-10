use std::collections::HashMap;

use pest_consume::{ match_nodes, Parser, Error };
use crate::{instructionset::{EarpInstructionSet, EarpInstructionSetIdentifier}, error::EarpAssemblerError};

#[derive(Parser)]
#[grammar = "opcodemap.pest"]
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

    fn identifier(input: Node) -> PestResult<EarpInstructionSet> {
        Ok(match_nodes!(input.into_children();
            [set(s),version(v)] => EarpInstructionSet::new(&EarpInstructionSetIdentifier(s.to_string(),v))
        ))
    }

    fn identifiers(input: Node) -> PestResult<Vec<EarpInstructionSet>> {
        Ok(match_nodes!(input.into_children();
            [identifier(id)..] => id.collect()
        ))
    }

    fn map_line(input: Node) -> PestResult<(u64,&str)> {
        Ok(match_nodes!(input.into_children();
            [opcode(p),name(n)] => (p,n)
        ))
    }

    fn section(input: Node) -> PestResult<Vec<EarpInstructionSet>> {
        let mut out = HashMap::new();
        let node = input.clone();
        match_nodes!(input.into_children();
            [identifiers(mut sets),map_line(lines)..] => { 
                for (opcode,name) in lines {
                    for set in &mut sets {
                        let set = out.entry(set.identifier().clone()).or_insert_with(|| EarpInstructionSet::new(set.identifier()));
                        set.add(name,opcode).map_err(|e| node.error(e.to_string()))?;
                    }
                }
                ()
            }
        );
        Ok(out.drain().map(|(_,v)| v).collect())
    }

    fn document(input: Node) -> PestResult<Vec<EarpInstructionSet>> {
        let mut out = HashMap::new();
        let node = input.clone();
        match_nodes!(input.into_children();
            [section(sections)..,_EOI] => { 
                for section in sections {
                    for in_set in &section {
                        let set = out.entry(in_set.identifier().clone()).or_insert_with(|| EarpInstructionSet::new(in_set.identifier()));
                        set.merge(in_set).map_err(|e| node.error(e))?;
                    }
                }
                ()
            }
        );
        Ok(out.drain().map(|(_,v)| v).collect())
    }
}

fn parse_opcode_map(map: &str) -> PestResult<Vec<EarpInstructionSet>> {
    let input = EarpOpcodeMapParser::parse(Rule::document, map)?.single()?;
    EarpOpcodeMapParser::document(input)
}


pub(crate) fn load_opcode_map(map: &str) -> Result<Vec<EarpInstructionSet>,EarpAssemblerError> {
    parse_opcode_map(map).map_err(|e| EarpAssemblerError::BadOpcodeMap(e.to_string()))
}

// effectively tested by tests in instructionset.rs
