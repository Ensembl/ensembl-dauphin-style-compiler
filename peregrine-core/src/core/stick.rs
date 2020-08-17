/* At the moment, sticks are just strings. They will probably become more elaborate.
 * This abstraction should make that transition easier.
 */

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub struct Stick(String);

impl Stick {
    fn new(name: &str) -> Stick {
        Stick(name.to_string())
    }
}