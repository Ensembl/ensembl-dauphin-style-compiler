#[macro_export]
macro_rules! looper_lifetime {
    ($looper:ident, $base:ident, {$root:ident, $root_type: ty}, [$({$var:ident,$typ:ty}),*] ) => {
        use std::slice::Iter;
        use std::iter::Cycle;

        struct $looper<'t> {
            $root: Iter<'t,$root_type>,
            $(
                $var: Cycle<Iter<'t,$typ>>
            ),*
        }

        impl<'t> $looper<'t> {
            fn new<'q: 't>(object: &'t $base<'q>) -> $looper<'t> {
                $looper {
                    $root: object.$root.iter(),
                    $(
                        $var: object.$var.iter().cycle()
                    ),*
                }
            }
        }

        impl<'t> Iterator for $looper<'t> {
            type Item = (&'t $root_type,$(&'t $typ),*);
        
            fn next(&mut self) -> Option<(&'t $root_type,$(&'t $typ),*)> {
                if let Some($root) = self.$root.next() {
                    Some((
                        $root,
                        $(
                            self.$var.next().unwrap()
                        ),*
                    ))
                } else {
                    None
                }
            }
        }        
   };
}