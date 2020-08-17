/* At the moment, focus objects are just strings. They will probably become more elaborate.
 * This abstraction should make that transition easier.
 */

#[derive(Clone,Debug,Eq,PartialEq,Hash)]
pub struct Focus(Option<String>);

impl Focus {
    fn new(name: Option<&str>) -> Focus {
        Focus(name.map(|x| x.to_string()))
    }
}