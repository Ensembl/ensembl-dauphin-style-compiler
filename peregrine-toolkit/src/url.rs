use std::fmt;

/* dead simple URL library with only the manipulations we need as the rust crate is bloaty (for our uses).
 * We store the URL unparsed and manipulations are on the unparsed URL. Ugly but small. This is hidden from the
 * user so we can revisit this later, if needed. If your URL is particularly crazy it may generate nonsense.
 * Don't do that or pass in user-supplied data. Annoying because this is where security issues cree pin. But
 * I did _try_ looking ofr a non-bloated replacement.
 */

//TODO unit test

 /* return ? or # of start of qs or fragment (whichever first) */
fn find_path_end(value: &str) -> Option<usize> {
    for (i,c) in value.chars().enumerate() {
        if c == '?' || c == '#' {
            return Some(i);
        }
    }
    None
}

 /* return ? or # of start of qs or fragment (whichever first) */
 fn find_qp_end(value: &str) ->Option<usize> {
    for (i,c) in value.chars().enumerate() {
        if c == '#' {
            return Some(i);
        }
    }
    None
}

fn split_at_path_end(value: &str) -> (String,String) {
    let mut first = value.to_string();
    if let Some(separator) = find_path_end(value) {
        let second = first.split_off(separator);
        (first,second)
    } else {
        (first,String::new())
    }
}

fn split_at_qp_end(value: &str) -> (String,String) {
    let mut first = value.to_string();
    if let Some(separator) = find_qp_end(value) {
        let second = first.split_off(separator);
        (first,second)
    } else {
        (first,String::new())
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct Url(String);

impl Url {
    pub fn parse(url: &str) -> anyhow::Result<Url> {
        Ok(Url(url.to_string()))
    }

    pub fn add_path_segment(&self, segment: &str) -> Url {
        let (mut out,after_path) = split_at_path_end(&self.0);
        out.push_str("/");
        out.push_str(segment);
        out.push_str(&after_path);
        Url(out)
    }

    pub fn add_query_parameter(&self, param: &str) -> Url {
        let (mut out,after_path) = split_at_qp_end(&self.0);
        out.push_str(if out.contains("?") { "&" } else {"?" });
        out.push_str(param);
        out.push_str(&after_path);
        Url(out)
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}