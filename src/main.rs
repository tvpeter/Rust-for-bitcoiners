// Add appropriate imports here
use text_colorizer::*;
use std::env;


fn main() {
    /*
    * Your code will be compiled with rustc and executed with two command line argunents
    * ceasar_cipher <message> <shift>
    * shift has to be parsed as u8 and it's range should be within 1 to 26
    * You have to handle all possible invalid inputs and print "Invalid Input" using println!
    * These will also be tested
    * If the input are valid printout the encrypted message
     */
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 {
        println!("{} Invalid input", "Error:".red().bold());
        std::process::exit(1);
    }

    let message = &args[0];
    let shift: i32 = match args[1].parse::<i32>() {
        Ok(n)  => n, //accept any number so long as it can be parsed as i32
        _ => {
            println!("{} Invalid Input", "Error:".red().bold());
            std::process::exit(1);
        }
    };
    
    let encrypted_message = caesar_cipher(message, shift);
    println!("{}", encrypted_message.green()); // Don not change this
    
}

fn shift_alphabet(c: u8, shift: i32) -> u8 {
    // Implement this function

    // Hints
    let a = 'a' as u8;
    let z = 'z' as u8;
    let capital_a = 'A' as u8;
    let capital_z = 'Z' as u8;

    let shift = shift.rem_euclid(26) as u8; 

    // Only apply shift if c is within a-z or A-Z, otherwise don't change it
    if c >= a && c <= z {
        (c - a + shift) % 26 + a
    } else if c >= capital_a && c <= capital_z {
        (c - capital_a + shift) % 26 + capital_a
    } else {
        c
    }
}

/// The ceasar_cipher should work for both upper case and lower case letters
/// other characters should be kept as it is
fn caesar_cipher(message: &str, shift: i32) -> String {
    // In rust &str is a wrapper over &[u8] which is a slice of bytes
    
    let bytes = message.as_bytes(); // Convert the message to a slice of bytes

    let mut encrypted_bytes = Vec::new(); // Create a new vector to store the encrypted bytes
    // for each byte apply the shift_alphabet function and collect them in encrypted_bytes
    // hint: use a for loop

    for &byte in bytes {
        encrypted_bytes.push(shift_alphabet(byte, shift));
    }

    let encrypted_message =String::from_utf8(encrypted_bytes);
    // hint: Read https://doc.rust-lang.org/std/string/struct.String.html
    
    encrypted_message.unwrap() // Return the encrypted message as a String
}

// Example tests are given below

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_lowercase() {
        let message = "abc";
        let shifted = caesar_cipher(message, 3);
        assert_eq!(shifted, "def");
    }

    #[test]
    fn test_with_uppercase() {
        let message = "XYZ";
        let shifted = caesar_cipher(message, 3);
        assert_eq!(shifted, "ABC");
    }

    #[test]
    fn test_with_wraparound() {
        let message = "wxyz";
        let shifted = caesar_cipher(message, 3);
        assert_eq!(shifted, "zabc");
    }

    #[test]
    fn test_with_negative_shift() {
        let message = "def";
        let shifted = caesar_cipher(message, -3);
        assert_eq!(shifted, "abc");
    }

    #[test]
    fn test_with_non_alphabetic_characters() {
        let message = "hello, world!";
        let shifted = caesar_cipher(message, 3);
        assert_eq!(shifted, "khoor, zruog!");
    }

    #[test]
    fn test_with_large_shift() {
        let message = "abc";
        let shifted = caesar_cipher(message, 29); // Equivalent to a shift of 3
        assert_eq!(shifted, "def");
    }

    #[test]
    fn test_with_zero_shift() {
        let message = "rust";
        let shifted = caesar_cipher(message, 0);
        assert_eq!(shifted, "rust");
    }

    #[test]
    fn test_shift_alphabet_a_neg1() {
        assert_eq!(shift_alphabet(97, -1), 122);
    }

    #[test]
    fn test_shift_alphabet_a_52() {
        assert_eq!(shift_alphabet(97, 52), 97);
    }

}
