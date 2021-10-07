use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use sha1::{Sha1, Digest};
use std::time::Instant;
use rayon::prelude::*;

const CHAIN_LEN: usize = 20000;
const NUM_CHAINS: usize = 40000;
const PASS_SIZE: usize = 3;

// fn str_to_int(s: Vec<u8>) -> u128 {
//     let mut num: u128 = 0;
//     println!("{:?}", s);
//     for i in 0..s.len() {
// 	let index: u8;
// 	if s[i]-48 < 10 {
// 	    index = s[i]-48;
// 	} else if s[i]-65 < 26 {
// 	    index = s[i]-65;
// 	} else {
// 	    index = s[i]-97;
// 	}
// 	num += ((62 * i) + index as usize) as u128;
//     }
//     num
// }

fn alpha_to_ascii(s: Vec<u8>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for i in 0..s.len() {
	if s[i] <= 9 {
	    out.push(s[i]+48)
	} else if s[i] <= 35 {
	    out.push(s[i]-10+65)
	} else {
	    out.push(s[i]-36+97)
	}
    }
    out
}

struct RainbowChain {
    // initial: u128,
    initial: Vec<u8>,
    last: Vec<u8>,
}

// Screw it, just reinitialize each time.
fn sha1_hash (s: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(s);
    hasher.finalize().to_vec()
}

impl RainbowChain {
    fn new() -> RainbowChain {
	let rng = thread_rng();
	RainbowChain {
	    initial: rng.sample_iter(&Alphanumeric).take(PASS_SIZE).collect(),
	    last: Vec::new(),
	}
    }
    fn forward(original: Vec<u8>) -> RainbowChain {
	let mut string: Vec<u8> = original.clone();
	let mut hash: Vec<u8> = sha1_hash(string);
	for i in 0..CHAIN_LEN/2 {
	    string = reduce(hash, i);
	    hash = sha1_hash(string);
	}
	RainbowChain {
	    initial: original,
	    last: hash,
	}
    }
}

// TODO Sketchy implementation
fn reduce(hash: Vec<u8>, i: usize) -> Vec<u8>
{
    let mut out: Vec<u8> = Vec::new();
    for j in 0..PASS_SIZE {
	out.push(((hash[j] as usize + i) % 62) as u8);
    }
    alpha_to_ascii(out)    
}

fn check_column(chains: &Vec<RainbowChain>, target: Vec<u8>) -> bool {
    for i in 0..NUM_CHAINS {
	if target == chains.get(i).unwrap().last {
	    let mut string: Vec<u8> = chains.get(i).unwrap().initial.clone();
	    let mut hash: Vec<u8> = sha1_hash(string.clone());
	    for i in 0..CHAIN_LEN/2 {
		string = reduce(hash, i);
		hash = sha1_hash(string.clone());
	    }
	    println!("[CRACKED (END)] {}", String::from_utf8(string).unwrap());
	    return true;
	}
    }
    return false;
}

fn check_rows(chains: &Vec<RainbowChain>, target: Vec<u8>) -> bool {
    let mut hash: Vec<u8> = target.clone();
    let mut string: Vec<u8>;
    for i in 0..CHAIN_LEN/2 {
	string = reduce(hash, i);
	hash = sha1_hash(string.clone());
	for j in 0..NUM_CHAINS {
	    if hash == chains.get(j).unwrap().last {
		let mut string2 = chains.get(j).unwrap().initial.clone();
		let mut hash2: Vec<u8> = sha1_hash(string2.clone());
		for k in 0..CHAIN_LEN/2 {
		    if hash2 == target.clone() {
			print!("| \x1b[32m✓\x1b[0m {}", String::from_utf8(string2).unwrap());
			return true;
		    }
		    string2 = reduce(hash2.clone(), k);
		    hash2 = sha1_hash(string2.clone());
		}
		return false;
	    }
	}
    }
    false
}

fn main() {
    println!("🌈 Generating {}x{} rainbow table...", CHAIN_LEN, NUM_CHAINS);
    let mut chains: Vec<RainbowChain> = Vec::new();
    let init = Instant::now();
    for _i in 0..NUM_CHAINS {
	chains.push(RainbowChain::new());
    }
    println!("| Initialized in {:?}.", init.elapsed());
    let gen = Instant::now();
    chains = chains.par_iter().map(|i| RainbowChain::forward(i.initial.clone())).collect();
    println!("└ Generated in {:?}.", gen.elapsed());
    let targets: Vec<Vec<u8>> = vec![
	vec![169,153,62,54,71,6,129,106,186,62,37,113,120,80,194,108,156,208,216,157],
	vec![27,163,110,98,0,100,14,221,6,101,69,34,250,17,55,200,199,43,79,176],
	vec![105,191,28,123,95,58,228,150,169,106,23,124,164,49,197,198,146,57,250,140],
    ];
    let length = targets.len();
    println!("\n🔨 Cracking passwords...");
    let mut correct = 0;
    for target in targets.into_iter() {
	let start = Instant::now();
	if check_column(&chains, target.clone()) == false {
	    // println!("{}", check_rows(&chains, target.clone()));
	    if !check_rows(&chains, target.clone()) {
		print!("| \x1b[31m✗\x1b[0m Not in table");
	    } else {
		correct += 1;
	    }
	}
	println!(" in {:?}", start.elapsed());
    }
    println!("└ Cracked {}/{} passwords!", correct, length);
    // for i in 0..NUM_CHAINS {
    // 	// Borrow checkers are fun.
	// println!("{} {}",
	// 	 String::from_utf8(chains.get(i).unwrap().initial.clone()).unwrap(),
	// 	 hex::encode(chains.get(i).unwrap().last.clone()));
    // 	chains.push(RainbowChain::new());
    // }
    // for i in 0..NUM_CHAINS {
    // 	String::from_utf8(chains.get(i).unwrap().initial.clone()).unwrap(),
    // 	hex::encode(chains.get(i).unwrap().last.clone()));
    // }
    
    
}
