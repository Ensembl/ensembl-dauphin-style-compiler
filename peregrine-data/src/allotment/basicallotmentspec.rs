pub(super) fn remove_bracketed(spec: &mut String, start: &str, end: &str) -> Option<String> {
    let mut depth = None;
    if let Some(start) = spec.find(start) {
        if let Some(end) = spec[(start+1)..].find(end).map(|x| x+start+1) {
            depth = Some(spec[(start+1)..end].to_string());
            let mut new_spec = spec[0..start].to_string();
            if spec.len() > end {
                new_spec.push_str(&spec[end+1..].to_string());
            }
            *spec = new_spec;
        }
    }
    depth
}

pub(super) fn remove_depth(spec: &mut String) -> i8 {
    remove_bracketed(spec,"[","]").map(|x| x.parse::<i8>().ok()).flatten().unwrap_or(0)
}

pub(super) fn remove_secondary(spec: &mut String) -> Option<String> {
    remove_bracketed(spec,"{","}")
}

pub(super) fn remove_group(spec: &mut String) -> Option<String> {
    remove_bracketed(spec,"\"","\"")
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct BasicAllotmentSpec {
    name: String,
    depth: i8,
    secondary: Option<String>,
    group: Option<String>
}

impl BasicAllotmentSpec {
    pub(crate) fn from_spec(spec: &str) -> BasicAllotmentSpec {
        let mut spec = spec.to_string();
        let depth = remove_depth(&mut spec);
        let secondary = remove_secondary(&mut spec);
        let group = remove_group(&mut spec);
        BasicAllotmentSpec { name: spec, depth, secondary, group }
    }
    
    pub(crate) fn with_name(&self, name: &str) -> BasicAllotmentSpec {
        let mut spec = self.clone();
        spec.name = name.to_string();
        spec
    }

    pub(crate) fn depthless(&self) -> BasicAllotmentSpec {
        let mut out = self.clone();
        out.depth = 0;
        out
    }

    pub(crate) fn group(&self) -> &Option<String> { &self.group }
    pub(crate) fn name(&self) -> &str { &self.name }
    pub(crate) fn depth(&self) -> i8 { self.depth }
    pub(crate) fn secondary(&self) -> &Option<String> { &self.secondary }
}
