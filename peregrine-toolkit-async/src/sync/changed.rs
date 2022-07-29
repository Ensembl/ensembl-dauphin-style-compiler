pub struct Changed<T: PartialEq> {
    reported: Option<T>,
    unreported: Option<T>
}

impl<T: PartialEq+std::fmt::Debug> Changed<T> where T: PartialEq {
    pub fn new() -> Changed<T> {
        Changed {
            reported: None,
            unreported: None
        }
    }

    pub fn set(&mut self, value: T) {
        self.unreported = Some(value);
    }

    pub fn peek(&self) -> (Option<&T>,Option<&T>) {
        (self.reported.as_ref(),self.unreported.as_ref().or(self.reported.as_ref()))
    }

    pub fn is_changed(&mut self) -> bool { 
        self.unreported.is_some() && self.unreported != self.reported
    }

    pub fn report(&mut self, reuse: bool) -> Option<&T> {
        let mut update = false;
        if let Some(unreported) = self.unreported.take() {
            update = true;
            if let Some(reported) = self.reported.as_ref() {
                if reported == &unreported { update = false; }
            }
            self.reported = Some(unreported);
        }
        if update || reuse {
            self.reported.as_ref()
        } else {
            None
        }
    }
}
