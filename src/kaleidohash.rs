use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::time::Instant;
use std::fmt;
use rayon::prelude::*;
use human_format::{Scales, Formatter};
use std::collections::HashSet;
use openssl::sha::sha1;

const HASH_SIZE: usize = 20;
type Hash = [u8; HASH_SIZE];

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
    last: Hash,
}

impl RainbowChain {    
    fn forward(original: Vec<u8>, len: usize, pass_size: usize) -> RainbowChain {	
	let mut string: Vec<u8> = original.clone();
	let mut hash: Hash = sha1(&string);
	for i in 0..len/2 {
	    string = reduce(hash, i, pass_size);
	    hash = sha1(&string);
	}
	RainbowChain {
	    initial: original,
	    last: hash,
	}
    }
}

// TODO Sketchy implementation
fn reduce(hash: Hash, i: usize, pass_size: usize) -> Vec<u8>
{
    let mut out: Vec<u8> = Vec::new();
    for j in 0..pass_size {
	out.push(((hash[j] as usize + i + j*2) % 62) as u8);
    }
    alpha_to_ascii(out)
}

struct RainbowMetadata {
    chain_len: usize,
    num_chains: usize,
    pass_size: usize,
}

struct RainbowTable {
    info: RainbowMetadata,
    chains: Vec<RainbowChain>,    
}

impl RainbowTable {
    fn new(chain_len: usize, num_chains: usize, pass_size: usize) -> RainbowTable {
	let mut r = RainbowTable {
	    info: RainbowMetadata {
		chain_len: chain_len,
		num_chains: num_chains,
		pass_size: pass_size,
	    },
	    chains: Vec::new(),
	};
	
	let mut initials: HashSet<Vec<u8>> = HashSet::new();    
	for _i in 0..r.info.num_chains {
	    loop {
		let plaintext: Vec<u8> = thread_rng()
		    .sample_iter(&Alphanumeric)
		    .take(r.info.pass_size)
		    .collect();
		if initials.insert(plaintext.clone()) {		    
		    r.chains.push(RainbowChain {initial: plaintext, last: [0; HASH_SIZE]});
		    break;
		}
	    }
	}
	r.chains = r.chains.par_iter()
	    .map(|i| RainbowChain::forward(i.initial.clone(),
					   r.info.chain_len,
					   r.info.pass_size))
	    .collect::<Vec<RainbowChain>>();
	for i in 0..5 {
	    println!("{}", hex::encode(r.chains.get(i).unwrap().last)) ;
	}
	r
    }

    fn check_column(&self, target: Hash) -> Option<String> {
	for i in 0..self.info.num_chains {
	    if target == self.chains.get(i).unwrap().last {
		let mut string: Vec<u8> = self.chains.get(i).unwrap().initial.clone();
		let mut hash: Hash = sha1(&string);
		for i in 0..self.info.chain_len/2 {
		    string = reduce(hash, i, self.info.pass_size);
		    hash = sha1(&string.clone());
		}
		return Some(String::from_utf8(string).unwrap());
	    }
	}
	return None;
    }     

    fn check_rows(&self, target: Hash) -> Option<String> {
	let mut hash: Hash = target.clone();
	let mut string: Vec<u8>;
	for i in 0..self.info.chain_len/2 {
	    string = reduce(hash, i, self.info.pass_size);
	    hash = sha1(&string.clone());	    
	    for j in 0..self.info.num_chains {
		if hash == self.chains.get(j).unwrap().last {
		    let mut string2 = self.chains.get(j).unwrap().initial.clone();
		    let mut hash2: Hash = sha1(&string2.clone());
		    for k in 0..self.info.chain_len/2 {
			if hash2 == target.clone() {
			    return Some(String::from_utf8(string2).unwrap());
			}
			string2 = reduce(hash2.clone(), k, self.info.pass_size);
			hash2 = sha1(&string2.clone());
		    }
		}
	    }
	}
	return None;
    }
    fn lookup(&self, target: Hash) -> Option<String> {
	if let Some(string) = self.check_column(target) {
	    return Some(string);
	} else {
	    return self.check_rows(target);	    
	}
    }
    fn duplicates(&self) -> u64 {
	let mut set: HashSet<Hash> = HashSet::new();
	let mut duplicates = 0;
	for i in 0..self.info.num_chains {
	    if set.insert(self.chains.get(i).unwrap().last) == false {
		duplicates+=1
	    }
	}
	return duplicates;
    }
}

impl fmt::Display for RainbowTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	let bytes = (HASH_SIZE+self.info.pass_size)*self.info.num_chains;
	let mut scales = Scales::new();
	scales.with_base(1024).with_suffixes(vec!["","Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi"]);
	let size = Formatter::new()
            .with_scales(scales)
            .with_units("B")
            .format(bytes as f64);
        write!(f, "{} {}x{} rainbow table with {} hashes.", size,
	       self.info.num_chains, self.info.chain_len, (self.info.chain_len/2)*self.info.num_chains)
    }
}

fn main() {
    let r: RainbowTable = RainbowTable::new(2000, 80000, 3);
    assert_eq!(sha1(b"abc"), [169,153,62,54,71,6,129,106,186,62,37,113,120,80,194,108,156,208,216,157]);
    println!("{}", r);
    println!("{}", r.duplicates());
    println!("{:#?}", r.lookup(sha1(b"abc")));
    println!("{:#?}", r.lookup(sha1(b"aaa")));    
    println!("{:#?}", r.lookup(sha1(b"zZz")));
    println!("{:#?}", r.lookup(sha1(b"a2c")));
    println!("{:#?}", r.lookup(sha1(b"baa")));    
    println!("{:#?}", r.lookup(sha1(b"zoz")));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_small() {
	let r: RainbowTable = RainbowTable::new(2000, 40000, 3);
	// assert_eq!(sha1(b"abc"), [169,153,62,54,71,6,129,106,186,62,37,113,120,80,194,108,156,208,216,157]);
	println!("{}", r);
	assert_eq!(r.lookup(sha1(b"abc")), Some(String::from("abc")));
    }

    
}
