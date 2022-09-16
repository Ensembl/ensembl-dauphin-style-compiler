use std::collections::HashMap;

#[derive(Clone,Hash,PartialEq,Eq,Debug)]
pub enum CoordinateSystemVariety {
    /* base = bp, tangent = x-px, normal = y-px (-ve = error!)
     * moves as user scrolls, optimised for bulk data
     */
    Tracking,

    /* base = bp, tangent = x-px, normal = y-px (-ve = bottom)
     * moves as user scrolls, vertically attached to viewport
     * identical intent to Tracking: less efficient but handles difficult cases (eg negative coordinates)
     */
    TrackingSpecial,

    /* base = 0->left-of-window, 1->right-of-window, tangent = x-px,  normal = y-px (-ve = bottom)
     * drawing relative to the window, attached vertically to viewport
     */
    Window,

    /* base = 0->left-of-window, 1->right-of-window, tangent = x-px,  normal = y-px (-ve = bottom)
     * drawing relative to the window, scrolls vertically with overflow
     */
    //Content,

    /* base = 0->top-of-window, 1->bottom-of-window, tangent = y-px,  normal = x-px (-ve = bottom)
     * drawing relative to the window on left and right
     */
    Sideways,

    /* Don't draw
     */
    Dustbin,
}

impl CoordinateSystemVariety {
    pub fn from_string(name: &str) -> CoordinateSystemVariety {
        match name {
            "tracking-special" => CoordinateSystemVariety::TrackingSpecial,
            "window" => CoordinateSystemVariety::Window,
            "sideways" => CoordinateSystemVariety::Sideways,
            "dustbin" => CoordinateSystemVariety::Dustbin,
            _ => CoordinateSystemVariety::Tracking
        }
    }
}

#[derive(Clone,Hash,PartialEq,Eq,Debug)]
pub struct CoordinateSystem(pub CoordinateSystemVariety,pub bool);

impl CoordinateSystem {
    pub fn build(spec: &HashMap<String,String>) -> (Option<CoordinateSystemVariety>,Option<bool>) {
        (
            spec.get("system").map(|coord_system| CoordinateSystemVariety::from_string(coord_system)),
            spec.get("direction").map(|x| x == "reverse")
        )
    }

    pub fn from_build(coord_system: Option<CoordinateSystemVariety>, reverse: Option<bool>) -> CoordinateSystem {
        let coord_system = coord_system.unwrap_or(CoordinateSystemVariety::Window);
        let reverse = reverse.unwrap_or(false);
        CoordinateSystem(coord_system,reverse)
    }

    pub fn is_dustbin(&self) -> bool {
        match self.0 {
            CoordinateSystemVariety::Dustbin => true,
            _ => false
        }
    }

    pub fn is_tracking(&self) -> bool {
        match self.0 {
            CoordinateSystemVariety::Tracking | CoordinateSystemVariety::TrackingSpecial => true,
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
