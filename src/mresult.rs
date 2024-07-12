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
