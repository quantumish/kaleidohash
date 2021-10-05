use rand::{thread_rng, Rng};
use rand::distributions::{Alphanumeric, Uniform, Standard};
use sha1::{Sha1, Digest};

const CHAIN_LEN: usize = 2000;
const NUM_CHAINS: usize = 40000;


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
	let original: Vec<u8> = rng.sample_iter(&Alphanumeric).take(5).collect();
	let mut string: Vec<u8> = original.clone();
	// print!("{}", String::from_utf8(string.clone()).unwrap());
	let mut hash: Vec<u8> = sha1_hash(string);
	// print!(" {}\n", hex::encode(hash.clone()));
	for i in 0..CHAIN_LEN/2 {
	    string = reduce(hash, i);
	    // print!("{}", String::from_utf8(string.clone()).unwrap());
	    hash = sha1_hash(string);
	    // print!(" {}\n", hex::encode(hash.clone()));
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
    for j in 0..5 {
	out.push(((hash[j] as usize + i) % 62) as u8);
    }
    alpha_to_ascii(out)    
}    

fn main() {
    let mut chains: Vec<RainbowChain> = Vec::new();
    println!("Generating rainbow table...");
    let bar = indicatif::ProgressBar::new(NUM_CHAINS as u64);
    for _i in 0..NUM_CHAINS {
	chains.push(RainbowChain::new());
	bar.inc(1);
    }    
    let target: Vec<u8> = vec![78,245,122,10,177,105,93,110,48,234,242,160,42,74,245,16,101,182,141,160];
    for i in 0..NUM_CHAINS {
	if target == chains.get(i).unwrap().last {
	    let mut string: Vec<u8> = chains.get(i).unwrap().initial.clone();
	    let mut hash: Vec<u8> = sha1_hash(string.clone());
	    for i in 0..CHAIN_LEN/2 {
		string = reduce(hash, i);
		hash = sha1_hash(string.clone());
	    }
	    println!("[CRACKED (END)] {}", String::from_utf8(string).unwrap());
	    std::process::exit(0);
	}
    }
   
    let mut hash: Vec<u8> = target;
    let mut string: Vec<u8>;
    for i in 0..CHAIN_LEN/2 {
	string = reduce(hash, i);
	hash = sha1_hash(string.clone());
	for j in 0..NUM_CHAINS {
	    if hash == chains.get(j).unwrap().last {
		println!("[CRACKED] {}", String::from_utf8(string).unwrap());
		std::process::exit(0);
	    }
	}
    }

    println!("[ERROR] Not in table!")
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
