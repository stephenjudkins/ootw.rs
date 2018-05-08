#[macro_use]
extern crate nom;
extern crate byteorder;
extern crate sha1;
extern crate regex;

mod unpack;
mod data;


fn main() {
  let list = data::load_mem_entries();
  for (entry, data) in list {
    let mut m = sha1::Sha1::new();
    m.update(&data);
    
    println!("{:?} / {:?} / {}", entry, data.len(), m.digest().to_string());
  }

}
