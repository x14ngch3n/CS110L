// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;
use std::iter::FromIterator;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    println!("random word: {}", secret_word);

    // Your code here! :)
    println!("Welcome to CS110L Hangman!");
    let mut guess_word = vec!['-'; secret_word.len()];
    let mut guessed_letters: Vec<char> = Vec::new();
    let mut guesses = NUM_INCORRECT_GUESSES;

    while guesses > 0 {
        // print status
        println!("The word so far is {}", String::from_iter(&guess_word));
        println!(
            "You have guessed the following letters: {}",
            String::from_iter(&guessed_letters)
        );
        println!("You have {} guesses left", guesses);
        print!("Please guess a letter: ");
        // get input char
        io::stdout().flush().expect("Error flushing out.");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Error reading line.");
        // parse char
        let guess_char = guess.chars().collect::<Vec<char>>()[0];
        if guess_char.is_ascii_alphabetic() {
            guessed_letters.push(guess_char);
        }
        // check if in secret word
        match secret_word.find(guess_char) {
            Some(loc) => {
                guess_word[loc] = guess_char;
            }
            None => {
                println!("Sorry, that letter is not in the word");
                guesses -= 1;
            }
        }
        print!("");
        // check if successfully guess
        if secret_word_chars == guess_word {
            break;
        }
    }

    match guesses {
        0 => {
            println!("Sorry, you ran out of guesses!");
        }
        _ => {
            println!(
                "Congratulations you guessed the secret word: {}",
                secret_word
            );
        }
    }
}
