use std::fmt::Display;

pub fn print_vec<T: Display>(words: &Vec<T>) {
    for word in words {
        println!("{word}")
    }
}
