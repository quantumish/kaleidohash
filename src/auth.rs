use sha1::{Sha1, Digest};
use clap::{AppSettings, Clap};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File};
use std::option::Option;

#[derive(Clap)]
#[clap(version = "1.0", author = "quantumish")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCmd,
}

#[derive(Clap)]
enum SubCmd {
    Add(Add),
    Auth(Auth),
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Add {
    #[clap(short)]
    username: String,
    #[clap(short)]
    password: String,    
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Auth {
    #[clap(short)]
    username: String,
    #[clap(short)]
    password: String,    
}

#[derive(Serialize, Deserialize)]
struct Database {
    data: HashMap<String, Vec<u8>>,
}

impl Database {
    fn new() -> Database {
	Database {
	    data: HashMap::new(),
	}
    }

    fn load_or_create() -> Database {
	match std::fs::read_to_string("auth.db") {
	    Ok(s) => serde_json::from_str(&s).unwrap(),
	    Err(_) => Database::new(),
	}
    }

    fn insert(mut self, username: String, password: Vec<u8>) {
	self.data.insert(username, password);
	serde_json::to_writer(File::create("auth.db").unwrap(), &self).unwrap();	
    }

    fn get(self, username: String) -> Vec<u8> {
	self.data.get(&username).unwrap().to_vec()
    }
}


fn main() {
    let mut hasher = Sha1::new();
    let opts: Opts = Opts::parse();
    let db: Database = Database::load_or_create();
    match opts.subcmd {
	SubCmd::Add(a) => {
	    hasher.update(a.password);
	    let hash = hasher.finalize();
	    db.insert(a.username, hash.to_vec());
	},
	SubCmd::Auth(a) => {
	    hasher.update(a.password);
	    let hash = hasher.finalize();
	    if db.get(a.username) == hash.to_vec() {
		println!("You have authenticated!");
	    } else {
		println!("I'm calling the cops!");
	    }
	},
    }
}
