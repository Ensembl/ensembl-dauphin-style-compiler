use std::collections::HashMap;

pub(crate) struct RelativeLabelContext {
    prev: HashMap<String,i64>,
    labels: HashMap<i64,HashMap<String,Option<i64>>>
}

impl RelativeLabelContext {
    pub(crate) fn new() -> RelativeLabelContext {
        RelativeLabelContext {
            prev: HashMap::new(),
            labels: HashMap::new(),
        }
    }

    fn entry(&mut self, position: i64) -> &mut HashMap<String,Option<i64>> {
        self.labels.entry(position).or_insert_with(|| HashMap::new())   
    }

    pub(crate) fn add_label(&mut self, position: i64, label: &str) {
        /* Avoid funny-business caused by double labels */
        if let Some(old_position) = self.prev.get(label) {
            if *old_position == position { return; }
        }
        /* A label 0 is added, meaning 0f was from last definition (or start) to this point, pointing here  */
        let start_of_fwd_ref_here = *self.prev.get(label).unwrap_or(&0);
        self.prev.insert(label.to_string(),position);
        self.entry(start_of_fwd_ref_here).insert(format!("{}f",label),Some(position));
        /* ... but no further */
        self.entry(position).insert(format!("{}f",label),None);
        /* ... 0r will refer to here from no on */
        self.entry(position).insert(format!("{}r",label), Some(position));
    }

    pub(crate) fn fix_labels(&self, current_position: i64, labels: &mut HashMap<String,i64>) {
        if let Some(changes) = self.labels.get(&current_position) {
            for (label,position) in changes.iter() {
                if let Some(position) = position {
                    labels.insert(label.to_string(),*position);
                } else {
                    labels.remove(label);
                }
            }    
        }
    }
}
