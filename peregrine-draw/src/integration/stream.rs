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

use blackbox::{ blackbox_log };
use dauphin_interp::runtime::{ Payload, PayloadFactory };
use dauphin_interp::{ StreamConnector, Stream };
#[cfg(blackbox)]
use commander::{ cdr_get_name };
#[cfg(blackbox)]
use web_sys::console;

pub struct WebStreamConnector();

impl WebStreamConnector {
    pub fn new() -> WebStreamConnector {
        WebStreamConnector()
    }
}

#[cfg(console)]
impl StreamConnector for WebStreamConnector {
    fn notice(&self, msg: &str) -> anyhow::Result<()> {
        console::log_1(&format!("{}\n",msg).into());
        blackbox_log!("notice","dauphin '{}': {}",cdr_get_name(),msg);
        Ok(())
    }

    fn warn(&self, msg: &str) -> anyhow::Result<()> {
        console::warn_1(&format!("{}\n",msg).into());
        blackbox_log!("warn","dauphin '{}': {}",cdr_get_name(),msg);
        Ok(())
    }

    fn error(&self, msg: &str) -> anyhow::Result<()> {
        console::error_1(&format!("{}\n",msg).into());
        blackbox_log!("error","dauphin '{}': {}",cdr_get_name(),msg);
        Ok(())
    }
}

#[cfg(not(console))]
#[allow(unused)]
impl StreamConnector for WebStreamConnector {
    fn notice(&self, msg: &str) -> anyhow::Result<()> {
        blackbox_log!("notice","dauphin '{}': {}",cdr_get_name(),msg);
        Ok(())
    }

    fn warn(&self, msg: &str) -> anyhow::Result<()> {
        blackbox_log!("warn","dauphin '{}': {}",cdr_get_name(),msg);
        Ok(())
    }

    fn error(&self, msg: &str) -> anyhow::Result<()> {
        blackbox_log!("error","dauphin '{}': {}",cdr_get_name(),msg);
        Ok(())
    }
}

pub struct WebStreamFactory {
}

impl WebStreamFactory {
    pub fn new() -> WebStreamFactory {
        WebStreamFactory{
        }
    }
}

impl PayloadFactory for WebStreamFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(Stream::new(Box::new(WebStreamConnector::new()),false,true))
    }
}