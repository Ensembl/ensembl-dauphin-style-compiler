use std::{collections::HashSet, str::Chars, iter::Peekable};
use peregrine_toolkit::error::Error;

struct TagPredLexer<'a>(Peekable<Chars<'a>>);

impl<'a> TagPredLexer<'a> {
    fn new(stream: &'a str) -> TagPredLexer<'a> {
        TagPredLexer(stream.chars().peekable())
    }

    const SPECIAL: &'static str = "()&!|#@";
}

impl<'a> Iterator for TagPredLexer<'a> {
    type Item = (String,bool);

    fn next(&mut self) -> Option<Self::Item> {
        /* eat whitespace */
        while let Some(c) = self.0.peek() {
            if c.is_whitespace() { self.0.next(); } else { break; }
        }
        if let Some(first) = self.0.peek().cloned() {
            if Self::SPECIAL.contains(first) {
                self.0.next();
                return Some((first.to_string(),true)); // special symbol
            }
        } else {
            return None; // EOF (maybe after whitespace)
        }
        let mut id = String::new();
        let mut bs = false;
        while let Some(c) = self.0.peek() {
            let regular = !c.is_whitespace() && !Self::SPECIAL.contains(*c) && *c != '\\';
            if regular || bs {
                id.push(*c); // more token
                bs = false;
            } else if *c == '\\' { 
                bs = true; // escaping bs
            } else { 
                break; // something else, so done
            }
            self.0.next();
        }
        if bs { return Some(("".to_string(),true)); } // error due to trailing bs
        return Some((id,false));
    }
}

#[derive(Clone)]
#[cfg_attr(any(debug_assertions,test),derive(Debug))]
pub(crate) enum TagPred {
    Not(Box<TagPred>),
    And(Box<TagPred>,Box<TagPred>),
    Or(Box<TagPred>,Box<TagPred>),
    Tag(String),
    Stick(String),
    All
}

impl TagPred {
    fn parse3<'a>(lexer: &mut Peekable<TagPredLexer<'a>>) -> Result<TagPred,()> {
        match lexer.next() {
            Some((x,true)) if x == "(" => { 
                let out = Self::parse(lexer)?;
                if let Some((x,true)) = lexer.next() { if x == ")" { return Ok(out) } }
            },
            Some((x,true)) if x == "#" => {
                if let Some((tag,false)) = lexer.next() { 
                    return Ok(TagPred::Tag(tag))
                }
            }
            Some((x,true)) if x == "@" => {
                if let Some((stick,false)) = lexer.next() {
                    return Ok(TagPred::Stick(stick))
                }
            },
            _ => {}
        }
        Err(())
    }

    fn parse2<'a>(lexer: &mut Peekable<TagPredLexer<'a>>) -> Result<TagPred,()> {
        Ok(match lexer.peek() {
            Some((x,true)) if x == "!" => { 
                lexer.next();
                TagPred::Not(Box::new(Self::parse2(lexer)?))
            }
            _ => {
                Self::parse3(lexer)?
            }
        })
    }

    fn parse1<'a>(lexer: &mut Peekable<TagPredLexer<'a>>) -> Result<TagPred,()> {
        let a = Self::parse2(lexer)?;
        match lexer.peek() {
            Some((x,true)) if x == "&" => { lexer.next(); },
            _ => { return Ok(a); },
        }
        let b = Self::parse1(lexer)?;
        return Ok(TagPred::And(Box::new(a),Box::new(b)))
    }
    
    fn parse<'a>(lexer: &mut Peekable<TagPredLexer<'a>>) -> Result<TagPred,()> {
        let a = Self::parse1(lexer)?;
        match lexer.peek() {
            Some((x,true)) if x == "|" => { lexer.next(); }
            _ => { return Ok(a); },
        }
        let b = Self::parse(lexer)?;
        return Ok(TagPred::Or(Box::new(a),Box::new(b)))
    }

    pub(crate) fn new(spec: &str) -> Result<TagPred,Error> {
        if spec.trim() == "" { return Ok(TagPred::All); }
        let mut lexer = TagPredLexer::new(spec).peekable();
        let mut out = Self::parse(&mut lexer);
        if lexer.next().is_some() { out = Err(()); }
        out.map_err(|_| Error::operr("parse error in tagpred"))
    }

    pub(crate) fn evaluate(&self, tags: &HashSet<String>, stick_id: &str) -> bool {
        match self {
            TagPred::Not(el) => !el.evaluate(tags,stick_id),
            TagPred::And(a,b) => a.evaluate(tags,stick_id) && b.evaluate(tags,stick_id),
            TagPred::Or(a,b) => a.evaluate(tags,stick_id) || b.evaluate(tags,stick_id),
            TagPred::Tag(tag) => tags.contains(tag),
            TagPred::Stick(s) => stick_id == s,
            TagPred::All => true
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_tagpred_parsing() {
        assert_eq!("Ok(Stick(\"hello\"))",format!("{:?}",TagPred::new("@hello")));
        assert!(TagPred::new("@@").is_err());
        assert!(TagPred::new("@(").is_err());
        assert!(TagPred::new("@#").is_err());
        assert!(TagPred::new("@").is_err());
        assert_eq!("Ok(Tag(\"hello\"))",format!("{:?}",TagPred::new("# hello")));
        assert_eq!("Ok(All)",format!("{:?}",TagPred::new("")));
        assert_eq!("Ok(All)",format!("{:?}",TagPred::new(" ")));
        println!("{:?}",TagPred::new("#he llo"));
        assert!(TagPred::new("#he llo").is_err());
        assert!(TagPred::new("hello").is_err());
        assert_eq!("Ok(Or(Tag(\"a\"), Tag(\"b\")))",format!("{:?}",TagPred::new("#a | #b")));
        assert_eq!("Ok(And(Tag(\"a\"), Not(Stick(\"b\"))))",format!("{:?}",TagPred::new("#a & !@b")));
        assert_eq!("Ok(Or(Tag(\"c\"), And(Tag(\"a\"), Stick(\"b\"))))",format!("{:?}",TagPred::new("#c | #a & @b")));
        assert_eq!("Ok(And(Or(Tag(\"c\"), Not(Tag(\"a\"))), Stick(\"b\")))",format!("{:?}",TagPred::new("(#c | !#a) & @b")));
        assert_eq!("Ok(Not(Not(Stick(\"hello\"))))",format!("{:?}",TagPred::new("!!@hello")));    
        assert_eq!("Ok(And(Tag(\"a\"), Not(Stick(\"@b\"))))",format!("{:?}",TagPred::new(r"#a & !@\@b")));
        assert_eq!("Ok(And(Tag(\"a\"), Not(Stick(\"\\\\b\"))))",format!("{:?}",TagPred::new(r"#a & !@\\b")));
        assert!(TagPred::new(r"#a & !@\\@b").is_err());
        assert!(TagPred::new(r"#a & !\@b").is_err());
        assert!(TagPred::new(r"\").is_err());
        assert!(TagPred::new(r"#a\").is_err());
        assert!(TagPred::new(r"\#a & !@\\@b").is_err());
        assert_eq!("Ok(And(Tag(\"a \"), Stick(\"b\")))",format!("{:?}",TagPred::new(r"#a\ & @b")));
        assert_eq!("Ok(And(Tag(\"a &\"), Stick(\"b\")))",format!("{:?}",TagPred::new(r"#a\ \&& @b")));
    }
}
