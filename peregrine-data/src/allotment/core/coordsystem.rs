use std::collections::HashMap;

#[derive(Clone,Hash,PartialEq,Eq,Debug)]
pub enum CoordinateSystem {
    /* base = bp, tangent = x-px, normal = y-px (-ve = error!)
     * moves as user scrolls, optimised for bulk data
     */
    Tracking,

    /* base = bp, tangent = x-px, normal = y-px (-ve = error!)
     * moves as user scrolls, scrolls vertically with content
     * identical intent to Tracking: less efficient but handles difficult cases (eg running text)
     */
    TrackingSpecial,

    /* base = bp, tangent = x-px, normal = y-px (-ve = bottom)
     * moves as user scrolls, vertically attached to viewport
     */
    TrackingWindow,

    /* base = 0->left-of-window, 1->right-of-window, tangent = x-px,  normal = y-px (-ve = bottom)
     * drawing relative to the window, attached vertically to viewport
     */
    Window,

    /* base = 0->left-of-window, 1->right-of-window, tangent = x-px,  normal = y-px (-ve = bottom)
     * drawing relative to the window, scrolls vertically with overflow
     */
    Content,

    /* base = 0->top-of-window, 1->bottom-of-window, tangent = y-px,  normal = x-px (-ve = bottom)
     * drawing relative to the window on left and right.
     * 
     * THough both can access left and right via negative coordinates, playingfield squeeze means
     * we need to keep track of where we are just for sideways types.
     */
    SidewaysLeft,
    SidewaysRight,

    /* Don't draw
     */
    Dustbin,
}

impl CoordinateSystem {
    pub fn from_string(name: &str) -> CoordinateSystem {
        match name {
            "tracking-special" => CoordinateSystem::TrackingSpecial,
            "tracking-window" => CoordinateSystem::TrackingWindow,
            "window" => CoordinateSystem::Window,
            "content" => CoordinateSystem::Content,
            "left" => CoordinateSystem::SidewaysLeft,
            "right" => CoordinateSystem::SidewaysRight,
            "dustbin" => CoordinateSystem::Dustbin,
            _ => CoordinateSystem::Tracking
        }
    }
}

impl CoordinateSystem {
    pub fn build(spec: &HashMap<String,String>) -> Option<CoordinateSystem> {
        spec.get("system").map(|coord_system| CoordinateSystem::from_string(coord_system))
    }

    pub fn from_build(coord_system: Option<CoordinateSystem>) -> CoordinateSystem {
        coord_system.unwrap_or(CoordinateSystem::Window)
    }

    pub fn is_dustbin(&self) -> bool {
        match self {
            CoordinateSystem::Dustbin => true,
            _ => false
        }
    }

    pub fn is_tracking(&self) -> bool {
        match self {
            CoordinateSystem::Tracking | CoordinateSystem::TrackingSpecial | CoordinateSystem::TrackingWindow => true,
            _ => false
        }
    }

    pub fn up_from_bottom(&self) -> bool {
        match self {
            CoordinateSystem::SidewaysRight => true,
            _ => false
        }
    }

    pub fn flip_xy(&self) -> bool {
        match self {
            CoordinateSystem::SidewaysLeft | CoordinateSystem::SidewaysRight => true,
            _ => false
        }
    }
}
