pub fn expand_by_repeating<X>(data: &mut Vec<X>, target_size: usize) where X: Clone {
    let orig_len = data.len();
    if orig_len == 0 { return; }
    while data.len() < target_size {
        let deficit = target_size - data.len();
        let this_time = orig_len.min(deficit);
        data.extend_from_within(0..this_time);
    }
}
