/* At the moment, tracks are just strings. They will probably become more elaborate.
 * This abstraction should make that transition easier.
 */

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub struct Track(String);

impl Track {
    pub fn new(name: &str) -> Track {
        Track(name.to_string())
    }
}