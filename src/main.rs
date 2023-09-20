use std::{
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

use hex_string::HexString;
use rand::random;
use sha2::{Digest, Sha256};
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Number of threads to use
    #[structopt(short, long, default_value = "10")]
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

pub fn worker(
    hash: String,
    difficulty: u16,
    leading_zeros: u8,
    should_terminate: Arc<AtomicBool>,
    tx: mpsc::Sender<(bool, Vec<u8>, f64)>,
) {
    let mut vector_hash: Vec<u8> = string_to_bytes(hash);

    let current_difficulty_hash = get_difficulty_hash(difficulty, leading_zeros);
    let nonce: [u8; 12] = random::<[u8; 12]>();
    let mut count = 0;

    let start_time = Instant::now();

    vector_hash[4..16].copy_from_slice(&nonce);

    while !should_terminate.load(Ordering::Relaxed) {
        // Get the hash
        let hasher = Sha256::new_with_prefix(&vector_hash);
        let hasher = Sha256::new_with_prefix(hasher.finalize());
        let hash = hasher.finalize().to_vec();

        count += 1;

        // If there's a solution, send and break
        if hash.le(&current_difficulty_hash) {
            println!("solution vector: {:?}", (vector_hash));
            println!("solution hash: {:?}", (hash));
            tx.send((
                true,
                vector_hash[4..20].to_vec(),
                1000000f64 * count as f64 / (Instant::now() - start_time).as_micros() as f64,
            ))
            .unwrap();
            break;
        }

        if count % 10000000 == 0 {
            tx.send((
                false,
                vector_hash[4..20].to_vec(),
                1000000f64 * count as f64 / (Instant::now() - start_time).as_micros() as f64,
            ))
            .unwrap();
        }
        // Increment nonce
        unsafe {
            // println!("Before: {:?}", vector_hash);
            let bytes_hash = vector_hash.as_mut_ptr().add(4);
            increment_nonce(bytes_hash);
            // println!("After:  {:?}", vector_hash);
        }
    }
}

fn fetch_datum() -> (String, u16, u8) {
    let hash = fs::read_to_string("datum.txt").expect("Couldn't read the file.");
    return (hash, 65535, 8);
}

fn post_nonce(nonce: Vec<u8>) {
    println!("Sending solution: {:?}", nonce);
    fs::write("submit.txt", HexString::from_bytes(&nonce).as_string())
        .expect("Could not write file.");
}

pub fn main() {
    // setup threading and global variables
    let opt = Opt::from_args();
    let mut solved = false;
    let num_threads = opt.threads;
    let mut hash_rate = 0f64;
    let (tx, rx) = mpsc::channel();
    let should_terminate = Arc::new(AtomicBool::new(false));

    // Get the initial state of the chain
    let (mut hash, mut difficulty, mut leading_zeros) = fetch_datum();
    // let mut hash = hash_string.as_bytes();

    loop {
        let mut handles = vec![];
        let (hash_update, _, _) = fetch_datum().clone();

        // if solved && hash_update == hash {
        //     continue;
        // }

        solved = false;

        println!("Beginning new solution cycle...");

        for i in 0..num_threads {
            let should_terminate = Arc::clone(&should_terminate);
            let tx = tx.clone();
            let clone_hash = hash.clone();
            let handle = thread::spawn(move || {
                worker(clone_hash, difficulty, leading_zeros, should_terminate, tx);
            });
            handles.push(handle);
        }
        loop {
            // Non-blocking check if a worker found a solution
            match rx.try_recv() {
                Ok(answer) => {
                    let (has_solution, nonce, hr) = answer;
                    if has_solution {
                        println!("Found a solution!");
                        post_nonce(nonce);
                        solved = true;
                        println!(
                            "Waiting for new datum. Hash rate last round: {:.3}MH/s.",
                            num_threads as f64 * hash_rate / 1000000f64
                        );
                        break;
                    } else {
                        hash_rate =
                            (hash_rate * (num_threads as f64 - 1f64) + hr) / num_threads as f64
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No worker has found a solution yet. Continue polling or doing other tasks.
                    let new_datum = fetch_datum().clone();
                    if new_datum.0 != hash {
                        println!(
                            "New datum, starting new round. Hash rate last round: {:.3}MH/s.",
                            num_threads as f64 * hash_rate / 1000000f64
                        );
                        hash = new_datum.0;
                        difficulty = new_datum.1;
                        leading_zeros = new_datum.2;
                        break;
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => panic!("Channel disconnected"),
            }
            thread::sleep(Duration::from_millis(100)); // Optional, to avoid busy-waiting
        }

        // Reset for the next iteration
        should_terminate.store(true, Ordering::Relaxed);
        for handle in handles {
            handle.join().unwrap();
        }
        should_terminate.store(false, Ordering::Relaxed);
    }
}
