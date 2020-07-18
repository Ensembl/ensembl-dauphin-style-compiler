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

use dauphin_interp::util::DauphinError;
use super::core::{ DocumentResolver, ResolverQuery };
use super::core::ResolverResult;

pub struct SearchResolver {
    templates: Vec<String>
}

impl SearchResolver {
    pub fn new(templates: &[String]) -> SearchResolver {
        SearchResolver {
            templates: templates.to_vec()
        }
    }
}

impl DocumentResolver for SearchResolver {
    fn resolve(&self, query: &ResolverQuery) -> anyhow::Result<ResolverResult> {
        let verbosity = query.resolver().config().get_verbose();
        let suffix = query.current_suffix();
        for template in &self.templates {
            let new_path = template.replace("*",suffix);
            let new_subquery = query.new_subquery(&new_path);
            match query.resolver().document_resolve(&new_subquery) {
                Ok(out) => { 
                    if verbosity > 0 {
                        print!("success {} -> {}\n",suffix,new_path);
                    }
                    return Ok(out);
                },
                Err(err) => { 
                    if verbosity > 1 {
                        print!("trying {} -> {} failed due to {}\n",suffix,new_path,err);
                    }
                }
            }
        }
        Err(DauphinError::floating(&format!("not found in search path. Use -v -v for more info on paths searched")))
    }
}
