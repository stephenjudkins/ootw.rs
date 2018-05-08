use nom::{be_u32,be_u8};

use std::fs::File;
use std::io::Read;
use std::iter::FromIterator;
use std::collections::HashMap;
use unpack;
use regex::Regex;

#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
  Sound, Music, Bitmap, Palette, Script, Vertices, Unknown
}

use std::fs;

#[derive(Debug, Clone, Copy)]
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

pub fn load_mem_entries() -> Vec<(MemEntry, Vec<u8>)> {
    let mut memlist_file = File::open("data/memlist.bin").unwrap();
    let mut buffer = Vec::new();
    memlist_file.read_to_end(&mut buffer).unwrap();
    let (_, mem_entries) = mem_list(&buffer).unwrap();

    let r = Regex::new(r"bank([0-9a-f]+)").unwrap();

    let bank_files = fs::read_dir("data").unwrap().filter_map(|e| {
        let entry = e.unwrap();
        let s = entry.file_name().into_string().unwrap();
        r.captures(&s).map(|c| {
            let id = u8::from_str_radix(c.get(1).unwrap().as_str(), 16).unwrap();
            let mut file = File::open(entry.path()).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            (id, buffer)
        })
    });

    let bank_files_by_name:HashMap<u8, Vec<u8>> = HashMap::from_iter(bank_files);

    return mem_entries.iter().map(|e| {
        let bank_contents = bank_files_by_name.get(&e.bank_num).unwrap();
        let packed = bank_contents[(e.bank_pos as usize)..((e.bank_pos + e.packed_size) as usize)].to_vec();
        assert!(packed.len() == e.packed_size as usize);
        let unpacked = if e.packed_size == e.unpacked_size { packed } else { unpack::unpack(packed) };
        (e.clone(), unpacked)
    }).collect();

}
