use std::sync::Arc;
use crate::{Allotment, DataMessage};

use super::{allotment::CoordinateSystem, allotmentrequest::{AllotmentRequestImpl}};

pub struct DustbinAllotmentRequest();

impl AllotmentRequestImpl for DustbinAllotmentRequest {
    fn name(&self) -> String { "".to_string() }
    fn is_dustbin(&self) -> bool { true }
    fn priority(&self) -> i64 { 0 }
    fn register_usage(&self, _max: i64) {}    
    fn coord_system(&self) -> CoordinateSystem { CoordinateSystem::Window }
    fn depth(&self) -> i8 { 0 }

    fn allotment(&self) -> Result<Allotment,DataMessage> {
        Err(DataMessage::AllotmentNotCreated(format!("attempt to display the dustbin!")))
    }

    fn up(self: Arc<Self>) -> Arc<dyn AllotmentRequestImpl> { self }
}
