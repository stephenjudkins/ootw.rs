
use byteorder::{BigEndian, ReadBytesExt};

pub fn unpack(contents: Vec<u8>) -> Vec<u8> {
    fn shift_bit(uc: &mut UnpackCtx, cf: u32)->u32 {
    	let r_cf = uc.bits & 1;
    	uc.bits >>= 1;
    	if cf != 0 {
    		uc.bits |= 0x80000000;
    	}
    	return r_cf;
    }

    fn next_bit(uc: &mut UnpackCtx)->u32 {
    	let mut cf = shift_bit(uc, 0);
    	if uc.bits == 0 {
    		uc.bits = read_uint32(&uc.contents, uc.src);
            if uc.src > 4 { uc.src -= 4 } else {uc.src = 0};
    		uc.crc ^= uc.bits;
    		cf = shift_bit(uc, 1);
    	}
    	return cf;
    }
    //
    fn get_bits(uc: &mut UnpackCtx, num_bits: u8) -> u16  {
    	let mut c = 0 as u16;
        for _ in 0..num_bits {
    		c <<= 1;
    		if next_bit(uc) != 0 {
    			c |= 1;
    		}
    	}
    	return c;
    }
    //
    fn helper_1(uc: &mut UnpackCtx, num_bits: u8, add_count: u8) {
    	let count = get_bits(uc, num_bits) + (add_count as u16) + 1;
    	uc.datasize -= count as u32;

        for _ in 0..count {
            let bits = get_bits(uc, 8) as u8;
            uc.dst.push(bits);
    	}
    }
    fn helper_2(uc: &mut UnpackCtx, num_bits: u8) {
    	let i = get_bits(uc, num_bits);
    	let count = uc.size + 1;
    	uc.datasize -= count as u32;

        for _ in 0 .. count {
            let byte = uc.dst[uc.dst.len() - (i as usize)];
            uc.dst.push(byte);
    	}
    }
    struct UnpackCtx {
        size: usize,
        datasize: u32,
        crc: u32,
        bits: u32,
        src: usize,
        contents: Vec<u8>,
        dst: Vec<u8>
    }

    fn read_uint32(vec: &Vec<u8>, pos: usize) -> u32 {
        use std::io::Cursor;
        let mut cursor = Cursor::new(vec);
        cursor.set_position(pos as u64);
        return cursor.read_u32::<BigEndian>().unwrap();
    }

    let crc1 = read_uint32(&contents, contents.len() - 8);
    let bits = read_uint32(&contents, contents.len() - 12);
    let crc = crc1 ^ bits;
    let datasize = read_uint32(&contents, contents.len() - 4);

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
    		if next_bit(&mut uc) == 0 {
    			uc.size = 1;
    			if next_bit(&mut uc) == 0 {
    				helper_1(&mut uc, 3, 0);
    			} else {
    				helper_2(&mut uc, 8);
    			}
    		} else {
    			let c = get_bits(&mut uc, 2);
    			if c == 3 {
    				helper_1(&mut uc, 8, 8);
    			} else if c < 2 {
    				uc.size = (c + 2) as usize;
    				helper_2(&mut uc, (c + 9) as u8);
    			} else {
    				uc.size = get_bits(&mut uc, 8) as usize;
    				helper_2(&mut uc, 12);
    			}
    		}
    }

    if uc.crc != 0 {
        panic!("Expected {} == 0", uc.crc);
    }



  uc.dst.reverse();
  return uc.dst;
}
