use crossbeam::channel;
use std::collections::VecDeque;
use std::io::BufRead;
use std::time::Instant;
use std::{env, process, thread};

/// Determines whether a number is prime. This function is taken from CS 110 factor.py.
///
/// You don't need to read or understand this code.
fn is_prime(num: u32) -> bool {
    if num <= 1 {
        return false;
    }
    for factor in 2..((num as f64).sqrt().floor() as u32) {
        if num % factor == 0 {
            return false;
        }
    }
    true
}

/// Determines the prime factors of a number and prints them to stdout. This function is taken
/// from CS 110 factor.py.
///
/// You don't need to read or understand this code.
fn factor_number(num: u32) {
    let start = Instant::now();

    if num == 1 || is_prime(num) {
        println!("{} = {} [time: {:?}]", num, num, start.elapsed());
        return;
    }

    let mut factors = Vec::new();
    let mut curr_num = num;
    for factor in 2..num {
        while curr_num % factor == 0 {
            factors.push(factor);
            curr_num /= factor;
        }
    }
    factors.sort();
    let factors_str = factors
        .into_iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(" * ");
    println!("{} = {} [time: {:?}]", num, factors_str, start.elapsed());
}

#[allow(unused)]
/// Returns a list of numbers supplied via argv.
fn get_input_numbers() -> VecDeque<u32> {
    let mut numbers = VecDeque::new();
    for arg in env::args().skip(1) {
        if let Ok(val) = arg.parse::<u32>() {
            numbers.push_back(val);
        } else {
            println!("{} is not a valid number", arg);
            process::exit(1);
        }
    }
    numbers
}

fn main() {
    let num_threads = num_cpus::get();
    println!("Farm starting on {} CPUs", num_threads);
    let start = Instant::now();

    // create channels and let every thread receive it
    let mut handles = Vec::new();
    let (sender, receiver) = channel::unbounded();
    for _ in 0..num_threads {
        let receiver_clone = receiver.clone();
        // wait until sender down, the recv() will unwrap()
        handles.push(thread::spawn(move || {
            while let Ok(num) = receiver_clone.recv() {
                factor_number(num);
            }
        }));
    }

    println!("Please input your number to factor, seperated by space:");
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        for val in line.unwrap().split(" ") {
            if let Ok(num) = val.parse::<u32>() {
                sender
                    .send(num)
                    .expect("Tried to write to channels, while there's no receivers");
            } else {
                println!("Failed to parse {} to u32", val);
            }
        }
    }

    // join all the threads you created
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Total execution time: {:?}", start.elapsed());
}
