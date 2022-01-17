use std::sync::Arc;

use crate::{AllotmentMetadata, allotment::core::{allotmentrequest::AllotmentRequestImpl, allotment::Transformer}};

#[derive(Clone)]
pub struct AllotmentBox {
    padding_top: i64,
    padding_bottom: i64,
    internal_height: i64,
    min_height: Option<i64>
}

impl AllotmentBox {
    pub fn new(metadata: &AllotmentMetadata, internal_height: i64) -> AllotmentBox {
        AllotmentBox {
            padding_top: metadata.get_i64("padding-top").unwrap_or(0),
            padding_bottom: metadata.get_i64("padding-bottom").unwrap_or(0),
            internal_height,
            min_height: metadata.get_i64("min-height")
        }
    }

    pub fn empty() -> AllotmentBox {
        AllotmentBox { padding_top: 0, padding_bottom: 0, internal_height: 0, min_height: None}
    }

    pub fn merge(&self, other: &AllotmentBox) -> AllotmentBox {
        let min_height = match (self.min_height,other.min_height) {
            (Some(x),Some(y)) => Some(x.max(y)),
            (Some(x),None) => Some(x),
            (None,Some(y)) => Some(y),
            (None,None) => None
        };
        AllotmentBox {
            padding_top: self.padding_top.max(other.padding_top),
            padding_bottom: self.padding_bottom.max(other.padding_bottom),
            internal_height: self.internal_height.max(other.internal_height),
            min_height
        }
    }

    pub fn merge_requests<'a,F,T>(&'a self, requests: F) -> AllotmentBox where F: Iterator<Item=&'a Arc<AllotmentRequestImpl<T>>>, T: Transformer + 'a {
        let mut out = self.clone();
        for request in requests {
            if !request.ghost() {
                out = out.merge(&AllotmentBox::new(request.metadata(),request.max_used()));
            }
        }
        out
    }

    pub fn top_space(&self) -> i64 { self.padding_top }

    pub fn height(&self) -> i64 {
        let mut height = self.internal_height + self.padding_top + self.padding_bottom;
        if let Some(min_height) = self.min_height {
            if height < min_height {
                height = min_height;
            }
        }
        height
    }
}
