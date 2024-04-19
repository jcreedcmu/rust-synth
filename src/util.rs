use std::error::Error;

pub type Mostly<T> = Result<T, Box<dyn Error>>;
