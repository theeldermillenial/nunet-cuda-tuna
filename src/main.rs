use std::{fs, path::Path, thread, time::Duration};

use hex_string::HexString;
use rand::random;
use sha2::{Digest, Sha256};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Number of threads to use
    #[structopt(short, long, default_value = "4")]
    threads: usize,
}

fn get_difficulty_hash(difficulty_number: u16, leading_zeros: u8) -> Vec<u8> {
    let mut difficulty_hash = [0_u8; 32];

    if leading_zeros % 2 == 0 {
        let byte_location = leading_zeros / 2;
        difficulty_hash[byte_location as usize] = (difficulty_number / 256) as u8;
        difficulty_hash[(byte_location + 1) as usize] = (difficulty_number % 256) as u8;
    } else {
        let byte_location = leading_zeros / 2;
        difficulty_hash[byte_location as usize] = (difficulty_number / 4096) as u8;
        difficulty_hash[(byte_location + 1) as usize] = ((difficulty_number / 16) % 4096) as u8;
        difficulty_hash[(byte_location + 2) as usize] = (difficulty_number % 16) as u8;
    }
    difficulty_hash.to_vec()
}

fn string_to_bytes(hex_string: String) -> Vec<u8> {
    let mut vector = Vec::new();

    for i in (0..hex_string.len()).step_by(2) {
        let byte_str = &hex_string[i..i + 2];
        if let Ok(byte) = u8::from_str_radix(byte_str, 16) {
            vector.push(byte);
        } else {
            vector.push(0); // Failed to parse hex value
        }
    }

    vector
}

pub fn increment_nonce(x: *mut u8) {
    // let raw_ptr = x.as_mut_ptr();
    let int_ptr: *mut i64 = x as *mut i64;
    unsafe {
        *int_ptr = *int_ptr + 1;
    }
}

pub fn worker(hash: String, difficulty: u16, leading_zeros: u8) -> Vec<u8> {
    let mut vector_hash: Vec<u8> = string_to_bytes(hash);

    let current_difficulty_hash = get_difficulty_hash(difficulty, leading_zeros);
    let nonce: [u8; 12] = random::<[u8; 12]>();

    vector_hash[4..16].copy_from_slice(&nonce);

    loop {
        // Get the hash
        let hasher = Sha256::new_with_prefix(&vector_hash);
        let hasher = Sha256::new_with_prefix(hasher.finalize());
        let hash = hasher.finalize().to_vec();

        // If there's a solution, send and break
        if hash.le(&current_difficulty_hash) {
            break;
        }
        // Increment nonce
        unsafe {
            let bytes_hash = vector_hash.as_mut_ptr().add(4);
            increment_nonce(bytes_hash);
        }
    }

    return vector_hash[4..20].to_vec();
}

fn fetch_datum() -> (String, u16, u8) {
    while !Path::new("datum.txt").exists() {
        thread::sleep(Duration::from_millis(100)); // Optional, to avoid busy-waiting
    }
    let hash = fs::read_to_string("datum.txt").expect("Couldn't read the file.");
    return (hash, 65535, 8);
}

fn post_nonce(nonce: Vec<u8>) {
    // println!("Sending solution: {:?}", nonce);
    fs::write("submit.txt", HexString::from_bytes(&nonce).as_string())
        .expect("Could not write file.");
}

pub fn main() {
    // Get the initial state of the chain
    let (mut hash, mut difficulty, mut leading_zeros) = fetch_datum();

    loop {
        let clone_hash = hash.clone();
        println!("hash: {}", hash);
        let nonce = worker(clone_hash, difficulty, leading_zeros);

        post_nonce(nonce);
        let new_datum = fetch_datum().clone();
        hash = new_datum.0;
        difficulty = new_datum.1;
        leading_zeros = new_datum.2;
    }
}
