use sha1::{Sha1, Digest};
fn main() {
    let mut hasher = Sha1::new();
    hasher.update(b"Testing");
    println!("{:?}", hex::encode(hasher.finalize()));
}
