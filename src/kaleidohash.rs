
use rand::{thread_rng, Rng, seq::SliceRandom};
use std::time::Instant;
use std::fmt;
use rayon::prelude::*;
use human_format::{Scales, Formatter};
use std::collections::HashSet;
use openssl::sha::sha1;
use std::sync::atomic::{AtomicU64, Ordering};
use indicatif::ProgressBar;

const HASH_SIZE: usize = 20;
type Hash = [u8; HASH_SIZE];

// Dummy struct to allow implementing rand::distributions::Distribution trait.
struct Charset;

// Implements std::rand::Distribution trait to allow Charset to be sampleable.
impl rand::distributions::Distribution<u8> for Charset {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u8 {
	*"0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz".to_string().into_bytes()
	    .choose(rng).unwrap()
    }
}

// Representation of rainbow chain's initial value and final hash.
struct RainbowChain {
    initial: Vec<u8>,
    last: Hash,
}

impl RainbowChain {
    // Returns a fully generated rainbow chain and updates the given progress bar accordingly.
    //
    // # Arguments 
    // * `original` - A vector of ASCII bytes representing the initial plaintext
    // * `len` - The length of the chain to generate
    // * `progress` - A indicatif::ProgressBar to update
    // * `n` - A counter (representing the number of chains generated) to increment
    //
    // # Examples
    // ```
    // use indicatif::ProgressBar;
    // use std::sync::atomic::AtomicU64;
    // let progress = ProgressBar::new(10);
    // let n = AtomicU64::new(0);
    // let chain = RainbowChain::forward(b"test".to_vec(), 10, &progress, n)
    // ```
    fn forward(original: Vec<u8>, len: usize, progress: &ProgressBar, n: &AtomicU64) -> RainbowChain {	
	let mut hash: Hash = sha1(&original);
	for _i in 0..len/2 {
	    hash = sha1(&reduce(&hash, original.len()));
	}
	let m = n.fetch_add(1, Ordering::Relaxed);	
        progress.set_position(m);
	RainbowChain {
	    initial: original,
	    last: hash,
	}
    }
}

// Returns a plaintext in the form of a vector of ASCII bytes generated from the given hash.
//
// # Arguments 
// * `hash` - A 20-byte SHA-1 hash.
// * `pass_size` - Size of plaintext to generate.
//
// # Panics
//
// Will panic for any `pass_size` > `HASH_SIZE`.
// 
// # Examples
// ```
// use openssl::sha::sha1;
// reduce(sha1(b"test"), 1);
// ```
fn reduce(hash: &Hash, pass_size: usize) -> Vec<u8> {
    let rng: rand_pcg::Pcg64 = rand_seeder::Seeder::from(hash).make_rng();
    rng.sample_iter(&Charset)
	.take(pass_size)
	.collect()	
}

// Struct representing information associated with rainbow table.
struct RainbowMetadata {
    chain_len: usize,
    num_chains: usize,
    pass_size: usize,
}

// Struct containing all rainbow chains and info.
struct RainbowTable {
    info: RainbowMetadata,
    chains: Vec<RainbowChain>,    
}

impl RainbowTable {
    // Returns a new rainbow table.
    //
    // # Arguments 
    // * `chain_len` - Length of each rainbow chain.
    // * `num_chains` - Number of chains to generate.
    // * `pass_size` - Size of plaintexts to generate.
    //
    // # Side Effects
    // Prints a progress bar to stdout.
    // 
    // # Examples
    // ```
    // let r = RainbowTable::new(100, 1000, 3);
    // ```
    fn new(chain_len: usize, num_chains: usize, pass_size: usize) -> RainbowTable {
	let mut table = RainbowTable {
	    info: RainbowMetadata {
		chain_len,
		num_chains,
		pass_size,
	    },
	    chains: Vec::new(),
	};

	// Keep trying to generate unique plaintext.
	let mut initials: HashSet<Vec<u8>> = HashSet::new();    
	for _i in 0..table.info.num_chains {
	    loop {
		let plaintext: Vec<u8> = ChaCha20Rng::from_entropy()
		    .sample_iter(&Charset)
		    .take(table.info.pass_size)
		    .collect();
		// Only push if unique
		if initials.insert(plaintext.clone()) {		    
		    table.chains.push(RainbowChain {initial: plaintext, last: [0; HASH_SIZE]});
		    break;
		}
	    }
	}

	let progress = ProgressBar::new(num_chains as u64);
        progress.enable_steady_tick(250);	
        let n = AtomicU64::new(0);

	// Generate chains in parallel
	table.chains = table.chains.par_iter()
	    .map(|i| RainbowChain::forward(i.initial.clone(),
					   table.info.chain_len,
					   &progress,
					   &n))
	    .collect::<Vec<RainbowChain>>();

	progress.finish();

	// Sort chains for very fast lookup
	table.chains.sort_by(|a,b| b.last.cmp(&a.last));
	table
    }

    // Returns the plaintext (if it exists) of a hash by checking final column of
    // table and recomputing chain.
    //
    // # Arguments 
    // * `hash` - Hash to search for.
    //
    // # Examples
    // ```
    // let r = RainbowTable::new(100, 1000, 3);
    // r.check_column(sha1(b"test"));
    // ```
    fn check_column(&self, target: Hash) -> Option<String> {
	// Find which final hash it matches (or if it matches none at all)
	let a = self.chains.binary_search_by(|i| target.cmp(&i.last));
	if let Ok(c) = a {
	    // Recompute chain to get preceding plaintext
	    let mut string: Vec<u8> = self.chains.get(c).unwrap().initial.clone();
	    let mut hash: Hash = sha1(&string);
	    for _i in 0..self.info.chain_len/2 {
		string = reduce(&hash, self.info.pass_size);
		hash = sha1(&string);
	    }
	    return Some(String::from_utf8(string).unwrap());
	}
	return None;
    }     

    // Returns the plaintext (if it exists) of a hash by applying reduction/hashes then comparing
    // to the hashes in table's final column (and recomputing to get plaintext if hash matches).
    //
    // # Arguments 
    // * `hash` - Hash to search for.
    //
    // # Examples
    // ```
    // let r = RainbowTable::new(100, 1000, 3);
    // r.check_rows(sha1(b"test"));
    // ```
    fn check_rows(&self, target: Hash) -> Option<String> {
	let mut hash: Hash = target;
	let mut string: Vec<u8>;
	// Reduce, hash, repeat until hash matches an end hash in the table
	for _i in 0..self.info.chain_len/2 {
	    string = reduce(&hash, self.info.pass_size);
	    hash = sha1(&string);
	    let a = self.chains.binary_search_by(|i| hash.cmp(&i.last));
	    if let Ok(c) = a {
		// Recompute matched chain
		let mut string2 = self.chains.get(c).unwrap().initial.clone();
		let mut hash2: Hash = sha1(&string2);
		for _k in 0..self.info.chain_len/2 {
		    if hash2 == target {
			return Some(String::from_utf8(string2).unwrap());
		    }
		    string2 = reduce(&hash2, self.info.pass_size);
		    hash2 = sha1(&string2);
		}
	    }
	}
	return None;
    }

    // Returns the plaintext (if it exists) of a hash by applying `check_columns()`
    // as well as `check_rows()`
    // 
    // # Arguments 
    // * `hash` - Hash to search for.
    //
    // # Examples
    // ```
    // let r = RainbowTable::new(100, 1000, 3);
    // r.lookup(sha1(b"test"));
    // ```
    fn lookup(&self, target: Hash) -> Option<String> {
	if let Some(string) = self.check_column(target) {
	    return Some(string);
	} else {
	    return self.check_rows(target);	    
	}
    }

    // Returns the number of duplicate in the rainbow table.
    // 
    // # Examples
    // ```
    // let r = RainbowTable::new(100, 1000, 3);
    // r.duplicates();
    // ```
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


// Implements the fmt::Display trait to allow RainbowTable to be printable with `println!()`.
// Prints a human readable summary of table info (rows, cols, size, hash count).
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
	       self.info.num_chains, self.info.chain_len, Formatter::new().format(((self.info.chain_len/2)*self.info.num_chains) as f64))
    }
}

fn main() {
    println!("Generating rainbow table...");
    let s = Instant::now();
    let r: RainbowTable = RainbowTable::new(400, 1500, 3);
    println!("{} in {:?}", r, s.elapsed());
    println!("{} duplicates out of {} rows.", r.duplicates(), r.info.num_chains);
    for _i in 0..100 {
	let target: Hash = sha1(&thread_rng()
				.sample_iter(&Charset)
				.take(r.info.pass_size)
				.collect::<Vec<u8>>());
	let start = Instant::now();
	match r.lookup(target.clone()) {
	    Some(s) => println!("Cracked {} in {:?}", s, start.elapsed()),
	    None => (),	    
	}
    }
    println!("Sanity check: cracked first column's final hash to get {:?}", r.lookup(sha1(&r.chains.get(0).unwrap().initial.clone())));
}
