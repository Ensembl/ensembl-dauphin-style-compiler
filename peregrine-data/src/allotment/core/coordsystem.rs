#[derive(Clone,Hash,PartialEq,Eq,Debug)]
pub enum CoordinateSystemVariety {
    /* base = bp, tangent = x-px, normal = y-px (-ve = error!)
     * moves as user scrolls, optimised for bulk data
     */
    Tracking,
    /* base = bp, tangent = x-px, normal = y-px (-ve = bottom)
     * moves as user scrolls,
     * identical intentto Tracking: less efficient but handles difficult cases (eg negative coordinates)
     */
    TrackingWindow,
    /* base = 0->left-of-winodw, 1->right-of-window, tangent = x-px,  normal = y-px (-ve = bottom)
     * drawing relative to the window
     */
    Window,
    /* base = 0->top-of-winodw, 1->bottom-of-window, tangent = y-px,  normal = x-px (-ve = bottom)
     * drawing relative to the window on left and right
     */
    Sideways,
}

#[derive(Clone,Hash,PartialEq,Eq,Debug)]
pub struct CoordinateSystem(pub CoordinateSystemVariety,pub bool);

impl CoordinateSystem {
    pub fn is_tracking(&self) -> bool {
        match self.0 {
            CoordinateSystemVariety::Tracking | CoordinateSystemVariety::TrackingWindow => true,
            _ => false
        }
    }

    pub fn secondary_stack(&self) -> bool { self.1 }

    pub fn negative_pixels(&self) -> bool { self.1 }

    pub fn up_from_bottom(&self) -> bool {
        match (&self.0,self.1) {
            (&CoordinateSystemVariety::Sideways,true) => true,
            _ => false
        }
    }

    pub fn flip_xy(&self) -> bool {
        match self.0 {
            CoordinateSystemVariety::Sideways => true,
            _ => false
        }
    }
}
