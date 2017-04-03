#[macro_use]
extern crate nom;

use nom::{IResult,be_u32,be_u8};
use nom::Err;

use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

extern crate byteorder;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};

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

fn unpack(contents: Vec<u8>) -> Vec<u8> {
    fn shiftBit(uc: &mut UnpackCtx, CF: u32)->u32 {
    	let rCF = uc.bits & 1;
    	uc.bits >>= 1;
    	if CF != 0 {
    		uc.bits |= 0x80000000;
    	}
    	return rCF;
    }

    fn nextBit(uc: &mut UnpackCtx)->u32 {
    	let mut CF = shiftBit(uc, 0);
    	if uc.bits == 0 {
    		uc.bits = READ_BE_UINT32(&uc.contents, uc.src);
            if uc.src > 4 { uc.src -= 4 } else {uc.src = 0};
    		uc.crc ^= uc.bits;
    		CF = shiftBit(uc, 1);
    	}
    	return CF;
    }
    //
    fn getBits(uc: &mut UnpackCtx, num_bits: u8) -> u16  {
    	let mut c = 0 as u16;
        for x in 0..num_bits {
    		c <<= 1;
    		if nextBit(uc) != 0 {
    			c |= 1;
    		}
    	}
    	return c;
    }
    //
    fn unpackHelper1(uc: &mut UnpackCtx, num_bits: u8, add_count: u8) {
    	let count = getBits(uc, num_bits) + (add_count as u16) + 1;
    	uc.datasize -= count as u32;

        for x in 0..count {
            let bits = getBits(uc, 8) as u8;
            uc.dst.push(bits);
    	}
    }
    fn unpackHelper2(uc: &mut UnpackCtx, num_bits: u8) {
    	let i = getBits(uc, num_bits);
    	let count = uc.size + 1;
    	uc.datasize -= count as u32;

        for x in 0 .. count {
            let byte = uc.dst[uc.dst.len() - (i as usize)];
            uc.dst.push(byte);
    	}
    }
    //

    struct UnpackCtx {
        size: usize,
        datasize: u32,
        crc: u32,
        bits: u32,
        src: usize,
        contents: Vec<u8>,
        dst: Vec<u8>
    }

    fn READ_BE_UINT32(vec: &Vec<u8>, pos: usize) -> u32 {
        use std::io::Cursor;
        let mut cursor = Cursor::new(vec);
        cursor.set_position(pos as u64);
        return cursor.read_u32::<BigEndian>().unwrap();
    }

    // bool delphine_unpack(uint8_t *dst, const uint8_t *src, int len) {
    let crc1 = READ_BE_UINT32(&contents, contents.len() - 8);
    let bits = READ_BE_UINT32(&contents, contents.len() - 12);
    let crc = crc1 ^ bits;
    let datasize = READ_BE_UINT32(&contents, contents.len() - 4);

    let out = Vec::with_capacity(datasize as usize);

        let mut uc = UnpackCtx {
            src: contents.len() - 16,
            datasize: datasize,
            size: 0,
            bits: bits,
            crc: crc,
            contents: contents,
            dst: out
        };

        while uc.datasize > 0 {
    		if nextBit(&mut uc) == 0 {
    			uc.size = 1;
    			if nextBit(&mut uc) == 0 {
    				unpackHelper1(&mut uc, 3, 0);
    			} else {
    				unpackHelper2(&mut uc, 8);
    			}
    		} else {
    			let c = getBits(&mut uc, 2);
    			if c == 3 {
    				unpackHelper1(&mut uc, 8, 8);
    			} else if c < 2 {
    				uc.size = (c + 2) as usize;
    				unpackHelper2(&mut uc, (c + 9) as u8);
    			} else {
    				uc.size = getBits(&mut uc, 8) as usize;
    				unpackHelper2(&mut uc, 12);
    			}
    		}
    }

    if uc.crc != 0 {
        panic!("Expected {} == 0", uc.crc);
    }



  uc.dst.reverse();
  return uc.dst;
}

fn contents(entry: &MemEntry) -> Vec<u8> {
  let bank_file_name = format!("data/bank{:02X}", entry.bank_num);
  let mut f = File::open(bank_file_name).unwrap();

  f.seek(SeekFrom::Start(entry.bank_pos as u64));

  let mut packed = vec![0; entry.packed_size as usize];
  f.read_exact(&mut packed).unwrap();

  if entry.packed_size == entry.unpacked_size {
    return packed;
  } else {
    return unpack(packed);
  }
}



extern crate sha1;

fn main() {
  let mut memlist_file = File::open("data/memlist.bin").unwrap();
  let mut buffer = Vec::new();
  memlist_file.read_to_end(&mut buffer).unwrap();

  let (_, list) = mem_list(&buffer).unwrap();

  for entry in list {
    println!("{:?}", entry);
    let data = contents(&entry);

    let mut m = sha1::Sha1::new();
    m.update(&data);

    println!("{:?} / {}", data.len(), m.digest().to_string());
  }

}
