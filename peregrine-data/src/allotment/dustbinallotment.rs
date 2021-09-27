use std::sync::Arc;
use crate::{Allotment, AllotmentGroup, DataMessage};

use super::allotmentrequest::{AllotmentRequestImpl};

pub struct DustbinAllotmentRequest();

impl AllotmentRequestImpl for DustbinAllotmentRequest {
    fn name(&self) -> String { "".to_string() }
    fn allotment_group(&self) -> AllotmentGroup { AllotmentGroup::Track }
    fn is_dustbin(&self) -> bool { true }
    fn priority(&self) -> i64 { 0 }

    fn allotment(&self) -> Result<Allotment,DataMessage> {
        Err(DataMessage::AllotmentNotCreated(format!("attempt to display the dustbin!")))
    }
    fn up(self: Arc<Self>) -> Arc<dyn AllotmentRequestImpl> { self }
}
