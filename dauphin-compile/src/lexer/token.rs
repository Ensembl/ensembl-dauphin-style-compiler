/* 
 *  See the NOTICE file distributed with this work for additional information
 *  regarding copyright ownership.
 *  
 *  Licensed under the Apache License, Version 2.0 (the "License"); you may 
 *  not use this file except in compliance with the License. You may obtain a
 *  copy of the License at http://www.apache.org/licenses/LICENSE-2.0
 *  
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS, WITHOUT 
 *  WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

#[derive(Debug,PartialEq,Clone)]
pub enum Token {
    Identifier(String),
    Number(String),
    Operator(String),
    Other(char),
    Error(String),
    LiteralString(String),
    LiteralBytes(Vec<u8>),
    FourDots,
    EndOfFile,
    EndOfLex
}

impl Token {
    pub fn name_for_errors(&self) -> String {
        match self {
            Token::Identifier(name) => { name.to_string() },
            Token::Number(number) => { number.to_string() },
            Token::Operator(oper) => { oper.to_string() },
            Token::Other(c) => { c.to_string() },
            Token::Error(error) => { error.to_string() },
            Token::LiteralString(s) => { s.to_string() },
            Token::LiteralBytes(_) => { "byte-literal".to_string() },
            Token::FourDots => { "::".to_string() },
            Token::EndOfFile => { "end of file".to_string() },
            Token::EndOfLex => { "end of file".to_string() }
        }
    }
}
