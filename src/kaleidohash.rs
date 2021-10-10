
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::time::Instant;
use std::fmt;
use rayon::prelude::*;
use human_format::{Scales, Formatter};
use std::collections::HashSet;
use openssl::sha::sha1;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Serialize, Deserialize};
use indicatif::ProgressBar;

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

fn alpha_to_u128(s: Vec<u8>) -> u128 {
    let mut out: u128 = 0;
    for i in 0..s.len() {
	if s[i] <= 48+9 && s[i] >= 48 {
	    println!("{} {}", s[i], s[i]-48);
	    out += (s[i]-48) as u128 * 62u128.pow(i as u32);
	} else if s[i] <= 65+26 && s[i] >= 65 {
	    println!("{} {}", s[i], s[i]-65+10);
	    out += (s[i]-65+10) as u128 * 62u128.pow(i as u32);
	} else if s[i] <= 97+26 && s[i] >= 97 {
	    println!("{} {}", s[i], s[i]-97+36);
	    out += (s[i]-97+36) as u128 * 62u128.pow(i as u32);
	}
    }
    out
}

#[derive(Serialize, Deserialize)]
struct RainbowChain {
    initial: Vec<u8>,
    last: Hash,
}

impl RainbowChain {    
    fn forward(original: Vec<u8>, len: usize, pass_size: usize, progress: &ProgressBar, n: &AtomicU64) -> RainbowChain {	
	let mut string: Vec<u8> = original.clone();
	let mut hash: Hash = sha1(&string);
	for i in 0..len/2 {
	    string = reduce(&hash, i);
	    hash = sha1(&string);
	}
	let m = n.fetch_add(1, Ordering::Relaxed);	
        progress.set_position(m);
	RainbowChain {
	    initial: original,
	    last: hash,
	}
    }
}

// TODO Sketchy implementation
fn reduce(hash: &Hash, deriv: usize) -> Vec<u8>
{
    alpha_to_ascii(hash.map(|i| (((i as u32 + deriv as u32) % (62))) as u8).to_vec())
    // let mut out: Vec<u8> = Vec::new();
    // for j in 0..pass_size {
    // 	out.push(((hash[j] as usize + i*j) % 62) as u8);
    // }
    // alpha_to_ascii(out)
}

#[derive(Serialize, Deserialize)]
struct RainbowMetadata {
    chain_len: usize,
    num_chains: usize,
    pass_size: usize,
}

#[derive(Serialize, Deserialize)]
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

	let progress = ProgressBar::new(num_chains as u64);
        progress.enable_steady_tick(250);	
        let n = AtomicU64::new(0);
	
	r.chains = r.chains.par_iter()
	    .map(|i| RainbowChain::forward(i.initial.clone(),
					   r.info.chain_len,
					   r.info.pass_size,
					   &progress,
					   &n))
	    .collect::<Vec<RainbowChain>>();

	progress.finish();
	// println!("{:#?}", r);
	r.chains.sort_by(|a,b| b.last.cmp(&a.last));
	// println!("{:#?}", r);
	r
    }

    fn check_column(&self, target: Hash) -> Option<String> {
	let a: Result<usize, usize> = self.chains.binary_search_by(|i| target.cmp(&i.last));
	// println!("{:#?}", a);
	if let Ok(c) = a {	    
	    let mut string: Vec<u8> = self.chains.get(c).unwrap().initial.clone();
	    let mut hash: Hash = sha1(&string);
	    for i in 0..self.info.chain_len/2 {
		string = reduce(&hash, i);
		hash = sha1(&string);
	    }
	    return Some(String::from_utf8(string).unwrap());
	}
	return None;
    }     

    fn check_rows(&self, target: Hash) -> Option<String> {
	let mut hash: Hash = target;
	let mut string: Vec<u8>;
	for i in 0..self.info.chain_len/2 {
	    string = reduce(&hash, i);
	    hash = sha1(&string);
	    let a: Result<usize, usize> = self.chains.binary_search_by(|i| target.cmp(&i.last));
	    println!("{:#?}", a);
	    if let Ok(c) = a {
		let mut string2 = self.chains.get(c).unwrap().initial.clone();
		let mut hash2: Hash = sha1(&string2);
		for k in 0..self.info.chain_len/2 {
		    if hash2 == target {
			return Some(String::from_utf8(string2).unwrap());
		    }
		    string2 = reduce(&hash2, k);
		    hash2 = sha1(&string2);
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
	for chain in &self.chains {
	    if set.insert(chain.last) == false {
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
    let r: RainbowTable = RainbowTable::new(5, 40000, 3);
    println!("{}", r);
    println!("{} duplicates out of {} rows.", r.duplicates(), r.info.num_chains);    
    println!("{:#?}", r.lookup(sha1(&r.chains.get(0).unwrap().initial)));
    println!("{:#?}", r.lookup(sha1(b"abc")));
    println!("{:#?}", r.lookup(sha1(b"123")));
    println!("{:#?}", r.lookup(sha1(b"whe")));
    println!("{:#?}", r.lookup(sha1(b"p2S")));
    println!("{:#?}", r.lookup(sha1(b"ZZc")));    
    println!("{:#?}", r.lookup(sha1(b"196")));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn u128_conv() {
	assert_eq!(alpha_to_u128(b"000".to_vec()), 0);
	assert_eq!(alpha_to_u128(b"aaa".to_vec()), 140652);
	assert_eq!(alpha_to_u128(b"Ab1".to_vec()), 6148);
    }    
}
