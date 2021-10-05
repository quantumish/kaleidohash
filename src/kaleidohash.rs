use rand::{thread_rng, Rng};
use rand::distributions::{Alphanumeric, Uniform, Standard};
use sha1::{Sha1, Digest};

const CHAIN_LEN: usize = 20;
const NUM_CHAINS: usize = 400;


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
// 	num += ((82 * i) + index as usize) as u128;
//     }
//     num
// }

fn alpha_to_ascii(s: Vec<u8>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for i in 0..s.len() {
	if s[i] <= 10 {
	    out.push(s[i]+48)
	} else if s[i] <= 36 {
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
    out.push(((hash[0] as usize + i) % 82) as u8);
    alpha_to_ascii(out)    
}    

fn main() {
    let mut chains: Vec<RainbowChain> = Vec::new();
    for _i in 0..NUM_CHAINS {
	chains.push(RainbowChain::new());
    }
    for i in 0..NUM_CHAINS {
	// Borrow checkers are fun.
	println!("{} {}",
		 String::from_utf8(chains.get(i).unwrap().initial.clone()).unwrap(),
		 hex::encode(chains.get(i).unwrap().last.clone()));
	chains.push(RainbowChain::new());
    }
    todo!("Crack some hashes!")
}
