#[macro_use]
extern crate nom;

use nom::{IResult,be_u32,be_u8};
use nom::Err;

use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub enum ResourceType {
  Sound, Music, Bitmap, Palette, Script, Vertices, Unknown
}

#[derive(Debug)]
pub struct MemEntry {
  pub tpe: ResourceType, // 0x1
  pub rank_num: u8, // 0x6
  pub bank_num: u8, // 0x7
  pub bank_pos: u32, // 0x8
  pub packed_size: u32, // 0xc
  pub unpacked_size: u32 // ox12
}

named!(resource_type<ResourceType>, alt!(
  map!(tag!(&[0x0]), |b| { ResourceType::Sound}) |
  map!(tag!(&[0x1]), |b| { ResourceType::Music}) |
  map!(tag!(&[0x2]), |b| { ResourceType::Bitmap}) |
  map!(tag!(&[0x3]), |b| { ResourceType::Palette}) |
  map!(tag!(&[0x4]), |b| { ResourceType::Script}) |
  map!(tag!(&[0x5]), |b| { ResourceType::Vertices}) |
  map!(tag!(&[0x6]), |b| { ResourceType::Unknown})
));

named!(mem_entry<MemEntry>, do_parse!(
  tpe: resource_type >>
  be_u32 >>
  rank_num: be_u8 >>
  bank_num: be_u8 >>
  bank_pos: be_u32 >>
  packed_size: be_u32 >>
  unpacked_size: be_u32 >> (MemEntry {
    tpe: tpe,
    rank_num: rank_num,
    bank_num: bank_num,
    bank_pos: bank_pos,
    packed_size: packed_size,
    unpacked_size: unpacked_size
  })
));



named!(mem_list<Vec<MemEntry>>,
  map!(
    many_till!(
      do_parse!(
        not!(tag!(&[0xFF])) >>
        be_u8 >>
        entry: mem_entry >>
        (entry)
      ),
      tag!(&[0xFF])
    ),
    |(l, _)| { l }
  )
);

fn main() {
  let mut memlist_file = File::open("data/memlist.bin").unwrap();
  let mut buffer = Vec::new();
  memlist_file.read_to_end(&mut buffer).unwrap();

  let (_, list) = mem_list(&buffer).unwrap();

  for entry in list {
    println!("{:?}", entry);
  }

}
