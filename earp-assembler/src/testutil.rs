pub fn no_error<T,E>(res: Result<T, E>) -> T where E: ToString {
    match res {
        Ok(v) => v,
        Err(e) => { 
            println!("unexpected error: {}",e.to_string());
            assert!(false);
            panic!();
        }
    }
}

pub fn yes_error<T,E>(res: Result<T, E>) -> E {
    match res {
        Ok(_) => {
            println!("expected error, didn't get one!");
            assert!(false);
            panic!();
        }
        Err(e) => e
    }
}
