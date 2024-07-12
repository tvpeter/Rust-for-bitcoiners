#![allow(unused)]

use std::fmt;
#[derive(Debug)]
enum MResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> MResult<T, E> {
    fn ok(value: T) -> Self {
           MResult::Ok(value)
    }
    // Function to create an Err variant
    fn err(error: E) -> Self {
       MResult::Err(error)
    }

    // Method to check if it's an Ok variant
    fn is_ok(&self) -> bool {
        match self {
            MResult::Ok(_) => true,
            MResult::Err(_) => false,
        }
    }

    // Method to check if it's an Err variant
    fn is_err(&self) -> bool {
        match self {
            MResult::Ok(_) => false,
            MResult::Err(_) => true,
        }
    }

    // Method to unwrap the Ok value, panics if it's an Err
    fn unwrap(self) -> T {
        match self {
            MResult::Ok(value) => value,
            MResult::Err(_) => panic!("called `unwrap` on an `Err` value"),
        }
    }

    // Method to unwrap the Err value, panics if it's an Ok
    fn unwrap_err(self) -> E {
        match self {
            MResult::Err(error) => error,
            MResult::Ok(_) => panic!("called `unwrap_err` on an `Ok` value")
        }
    }
}

// Add unit tests below
#[cfg(test)]
mod test {


    use super::*;

    #[test]
    fn test_ok(){
       let value = 10;
       let result: MResult<i32, &str> = MResult::ok(value);

        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_err(){
        let message = "error";
        let result: MResult<i32, &str> = MResult::err(message);

        let output = match result {
            MResult::Ok(_) => panic!("expected error variant"),
            MResult::Err(err) => err,
        };

        assert_eq!(output, message);
    }

    #[test]
    fn test_is_ok(){
        let result: MResult<i32, &str> = MResult::ok(10);
        assert!(result.is_ok());

        let result_err: MResult<i32, &str> = MResult::err("error");
        assert!(!result_err.is_ok());
    }

    #[test]
    fn test_is_err() {
        let result: MResult<i32, &str> = MResult::err("error");

        assert!(result.is_err());

        let result_ok: MResult<i32, &str> = MResult::ok(10);
        assert!(!result_ok.is_err());
    }

    #[test]
    #[should_panic(expected = "called `unwrap` on an `Err` value")]
    fn test_unwrap(){
        let value = 10;
        let result: MResult<i32, &str> = MResult::ok(value);
 
         assert_eq!(result.unwrap(), value);

        let erro_result: MResult<i32, &str> = MResult::err("error here");
        erro_result.unwrap();
    }

    #[test]
    #[should_panic(expected = "called `unwrap_err` on an `Ok` value")]
    fn test_unwrap_err() {
        let result: MResult<i32, &str> = MResult::err("error here");
        assert_eq!(result.unwrap_err(), "error here");

        let result: MResult<i32, &str> = MResult::ok(42);
        result.unwrap_err(); 
    }
}
