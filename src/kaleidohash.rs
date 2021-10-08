use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::time::Instant;
use rayon::prelude::*;
use human_format::{Scales, Formatter};
use std::collections::HashSet;
use openssl::sha::sha1;

const CHAIN_LEN: usize = 4000;
const NUM_CHAINS: usize = 2000000;
const PASS_SIZE: usize = 6;
const HASH_SIZE: usize = 20;

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
    last: [u8; HASH_SIZE],
}

impl RainbowChain {    
    fn forward(original: Vec<u8>) -> RainbowChain {	
	let mut string: Vec<u8> = original.clone();
	let mut hash: [u8; HASH_SIZE] = sha1(&string);
	for i in 0..CHAIN_LEN/2 {
	    string = reduce(hash, i);
	    // println!("{} {}", String::from_utf8(string.clone()).unwrap(), i);
	    hash = sha1(&string);
	}
	RainbowChain {
	    initial: original,
	    last: hash,
	}
    }
}

// TODO Sketchy implementation
fn reduce(hash: [u8; HASH_SIZE], i: usize) -> Vec<u8>
{
    let mut out: Vec<u8> = Vec::new();
    for j in 0..PASS_SIZE {
	out.push(((hash[j] as usize + i + j*2) % 62) as u8);
    }
    alpha_to_ascii(out)    
}

fn check_column(chains: &Vec<RainbowChain>, target: [u8; HASH_SIZE]) -> bool {
    for i in 0..NUM_CHAINS {
	if target == chains.get(i).unwrap().last {
	    let mut string: Vec<u8> = chains.get(i).unwrap().initial.clone();
	    let mut hash: [u8; HASH_SIZE] = sha1(&string.clone());
	    for i in 0..CHAIN_LEN/2 {
		string = reduce(hash, i);
		hash = sha1(&string.clone());
	    }
	    println!("[CRACKED (END)] {}", String::from_utf8(string).unwrap());
	    return true;
	}
    }
    return false;
}

fn initialize_chains(chains: &mut Vec<RainbowChain>) {    
    let mut initials: HashSet<Vec<u8>> = HashSet::new();    
    for _i in 0..NUM_CHAINS {
	loop {
	    let rng = thread_rng();
	    let plaintext: Vec<u8> = rng.sample_iter(&Alphanumeric).take(PASS_SIZE).collect();
	    if initials.insert(plaintext.clone()) {
		chains.push(RainbowChain {initial: plaintext, last: [0; HASH_SIZE]});
		break;
	    }
	}
    }
}

fn check_rows(chains: &Vec<RainbowChain>, target: [u8; HASH_SIZE]) -> bool {
    let mut hash: [u8; HASH_SIZE] = target.clone();
    let mut string: Vec<u8>;
    for i in 0..CHAIN_LEN/2 {
	string = reduce(hash, i);
	hash = sha1(&string.clone());
	for j in 0..NUM_CHAINS {
	    if hash == chains.get(j).unwrap().last {
		let mut string2 = chains.get(j).unwrap().initial.clone();
		let mut hash2: [u8; HASH_SIZE] = sha1(&string2.clone());
		for k in 0..CHAIN_LEN/2 {
		    if hash2 == target.clone() {
			print!("| \x1b[32m✓\x1b[0m {}", String::from_utf8(string2).unwrap());
			return true;
		    }
		    string2 = reduce(hash2.clone(), k);
		    hash2 = sha1(&string2.clone());
		}
	    }
	}
    }
    false
}

fn main() {    
    println!("🌈 Generating {}x{} rainbow table...", CHAIN_LEN, NUM_CHAINS);
    let mut chains: Vec<RainbowChain> = Vec::new();
    let init = Instant::now();
    initialize_chains(&mut chains);    
    println!("| Initialized in {:?}.", init.elapsed());
    let gen = Instant::now();
    chains = chains.par_iter().map(|i| RainbowChain::forward(i.initial.clone())).collect();
    let bytes = (HASH_SIZE+PASS_SIZE)*NUM_CHAINS;   
    let mut scales = Scales::new();
    scales
        .with_base(1024)
        .with_suffixes(vec!["","Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi"]);

    let size = Formatter::new()
               .with_scales(scales)
               .with_units("B")
               .format(bytes as f64);
    println!("└ Generated a {} table with {} hashes ({}% decrease) in {:?}.",
	     size,
	     Formatter::new().format(((CHAIN_LEN/2)*NUM_CHAINS) as f64),
	     100f64 - ((bytes as f64)/((((CHAIN_LEN/2)*NUM_CHAINS)*(PASS_SIZE+HASH_SIZE)) as f64)*100f64),
	     gen.elapsed());

    let mut set: HashSet<[u8; HASH_SIZE]> = HashSet::new();
    let mut duplicates = 0;
    for i in 0..NUM_CHAINS {
	if set.insert(chains.get(i).unwrap().last.clone()) == false {
	    duplicates+=1
	}
    }
    println!("{} duplicate end values out of {} rows.", duplicates, NUM_CHAINS);
    
    
    let targets: Vec<[u8;HASH_SIZE]> = vec![
	[48,39,76,71,144,59,209,186,199,99,59,191,9,116,49,73,235,171,128,95]
    ];
    println!("\n🔨 Cracking passwords...");
    let length = targets.len();
    let mut correct = 0;
    for target in targets.into_iter() {
	let start = Instant::now();
	if check_column(&chains, target.clone()) == false {
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
