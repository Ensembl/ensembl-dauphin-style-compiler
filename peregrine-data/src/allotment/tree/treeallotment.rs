use crate::allotment::core::allotmentrequest::AllotmentRequestImpl;

pub fn tree_best_offset<T>(this: &AllotmentRequestImpl<T>, offset: i64) -> i64 {
    let metadata = this.metadata();
    let padding_top = metadata.get_i64("padding-top").unwrap_or(0);
    offset + padding_top
}

pub fn tree_best_height<T>(this: &AllotmentRequestImpl<T>) -> i64 {
    let metadata = this.metadata();
    let mut height = this.max_used().max(0);
    if let Some(padding_top) = metadata.get_i64("padding-top") {
        height += padding_top;
    }
    if let Some(padding_bottom) = metadata.get_i64("padding-bottom") {
        height += padding_bottom;
    }
    if let Some(min_height) = metadata.get_i64("min-height") {
        if height < min_height {
            height = min_height;
        }
    }
    height
}
