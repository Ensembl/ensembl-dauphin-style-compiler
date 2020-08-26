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

use anyhow;
use serde_cbor::Value as CborValue;
use crate::runtime::InterpContext;
use std::future::Future;
use std::pin::Pin;

pub trait CommandDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>>;
    fn deserialize(&self, opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>>;
}

pub struct AsyncBlock<'a> {
    f: Box<dyn FnOnce(&'a mut InterpContext) -> Pin<Box<dyn Future<Output=anyhow::Result<()>> + 'a>>>
}

impl<'a> AsyncBlock<'a> {
    pub fn new(f: Box<dyn FnOnce(&'a mut InterpContext) -> Pin<Box<dyn Future<Output=anyhow::Result<()>> + 'a>>>) -> AsyncBlock {
        AsyncBlock { f }
    }

    pub async fn execute(self, context: &'a mut InterpContext) -> anyhow::Result<()> {
        (self.f)(context).await
    }
}

pub enum CommandResult<'a> {
    SyncResult(),
    AsyncResult(AsyncBlock<'a>)
}

pub trait InterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult>;
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct CommandTypeId(pub usize);
