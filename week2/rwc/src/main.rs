use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    let file = File::open(filename).unwrap();
    let (mut linen, mut wordn, mut charn) = (0, 0, 0);
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        linen += 1;
        wordn += line.split_whitespace().count();
        charn += line.chars().count();
    }
    println!("\tlines\twords\t characters");
    println!("counts\t{}\t{}\t{}", linen, wordn, charn);
}
