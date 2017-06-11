pub const CRC_POLY: u32 = 0xEDB88320;

pub fn checksum_ieee_four_byte(bytes: &[u8]) -> u32 {
    const TABLE_SIZE: usize = 0x100;
    const MAX_SLICE: usize = 4;
    let mut table: [[u32; TABLE_SIZE]; MAX_SLICE] = [[0; TABLE_SIZE]; MAX_SLICE];

    // build the table
    for i in 0..TABLE_SIZE as u32 {
        let mut crc: u32 = i;
        for _ in 0..8 {
            crc = (crc >> 1) ^ ((-((crc & 1) as i32)) as u32 & CRC_POLY);
        }
        table[0][i as usize] = crc;
    }

    for i in 0..TABLE_SIZE as u32 {
        // println!("{} crc {:#x}", i, crc);
        // / http://sourceforge.net/projects/slicing-by-8/
        // for (int slice = 1; slice < MaxSlice; slice++)
        //  Crc32Lookup[slice][i] = (Crc32Lookup[slice - 1][i] >> 8) ^
        //            Crc32Lookup[0][Crc32Lookup[slice - 1][i] & 0xFF];

        for slice in 1..MAX_SLICE {
            let i: usize = i as usize;
            let slice_index: usize = slice - 1;
            let x: u32 = table[slice_index][i] >> 8;
            let y: u32 = table[0][(table[slice_index][i] & 0xFF) as usize];
            table[slice][i] = x ^ y;
        }
    }


    // for i in 0..MAX_SLICE {
    // println!("table {}", i);
    // for j in 0..TABLE_SIZE / 8 {
    // println!("{:#010X},{:#010X},{:#010X},{:#010X}, {:#010x},{:#010x},{:#010x},{:#010x}",
    // table[i][j * 8 + 0],
    // table[i][j * 8 + 1],
    // table[i][j * 8 + 2],
    // table[i][j * 8 + 3],
    // table[i][j * 8 + 4],
    // table[i][j * 8 + 5],
    // table[i][j * 8 + 6],
    // table[i][j * 8 + 7]);
    // }
    // }
    //
    //
    // use the table
    //
    // const uint32_t* current = (const uint32_t*) data;
    //
    // process four bytes at once (Slicing-by-4)
    // while (length >= 4)
    // {
    // uint32_t one = *current++ ^ crc;
    // crc = Crc32Lookup[0][(one>>24) & 0xFF] ^
    // Crc32Lookup[1][(one>>16) & 0xFF] ^
    // Crc32Lookup[2][(one>> 8) & 0xFF] ^
    // Crc32Lookup[3][ one      & 0xFF];
    //
    // length -= 4;
    // }
    //
    // const uint8_t* currentChar = (const uint8_t*) current;
    // remaining 1 to 3 bytes (standard algorithm)
    // while (length--)
    // crc = (crc >> 8) ^ Crc32Lookup[0][(crc & 0xFF) ^ *currentChar++];
    //
    // return ~crc; // same as crc ^ 0xFFFFFFFF
    //


    // I think it might be better to iterate over 4 bytes manually vs trying to use an iterator
    let mut crc: u32 = !0;
    let mut length = bytes.len();
    for current in bytes.chunks(4) {
        // handle any bytes less than the chunksize
        if length < 4 {
            for &byte in current {
                crc = table[0][((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
                length -= 1;
            }
            break;
        }
        // reconstruct a u32 from bytes
        let b0: u32 = current[0] as u32;
        let b1: u32 = (current[1] as u32) << 8;
        let b2: u32 = (current[2] as u32) << 16;
        let b3: u32 = (current[3] as u32) << 24;
        // println!("or u32 {:#x}", (b0 | b1 | b2 | b3) ^ crc);
        let one: u32 = (b0 | b1 | b2 | b3) ^ crc;
        //   ((current[0] as u32) as u32 |
        // ((current[1] as u32) << 8) as u32 |
        // ((current[2] as u32) << 16) as u32 |
        // ((current[3] as u32) << 24) as u32) ^
        // crc;
        //
        // println!("reconstructed u32 {:#x}", one);
        let xb0: u8 = (one >> 24 & 0xFF) as u8;
        let xb1: u8 = (one >> 16 & 0xFF) as u8;
        let xb2: u8 = (one >> 8 & 0xFF) as u8;
        let xb3: u8 = (one >> 0 & 0xFF) as u8;
        let tl0: u32 = table[0][xb0 as usize];
        let tl1: u32 = table[1][xb1 as usize];
        let tl2: u32 = table[2][xb2 as usize];
        let tl3: u32 = table[3][xb3 as usize];

        // let tl0: u32 = table[0][(one >> 24 & 0xFF) as usize];
        // let tl1: u32 = table[1][(one >> 16 & 0xFF) as usize];
        // let tl2: u32 = table[2][(one >> 8 & 0xFF) as usize];
        // let tl3: u32 = table[3][(one >> 0 & 0xFF) as usize];
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b0, xb0, tl0);
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b1, xb1, tl1);
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b2, xb2, tl2);
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b3, xb3, tl3);
        crc = tl0 ^ tl1 ^ tl2 ^ tl3;

        // update how many bytes there are left to crc
        length -= 4;
        // println!("crc {:#010x}", crc);
    }

    assert!(length == 0);
    // i == current
    // crc = table[0][((crc ^ current as u32) & 0xFF) as usize] ^ (crc >> 8);
    // on exit do the standard 0xFFFF_FFFF flip
    //    println!("final crc {:#010x}", !crc);
    !crc
}


pub fn checksum_ieee_sixteen_byte(bytes: &[u8]) -> u32 {
    const TABLE_SIZE: usize = 0x100;
    const SLICE_SIZE: usize = 16;
    let mut table: [[u32; TABLE_SIZE]; SLICE_SIZE] = [[0; TABLE_SIZE]; SLICE_SIZE];

    // build the table
    for i in 0..TABLE_SIZE as u32 {
        let mut crc: u32 = i;
        for _ in 0..8 {
            crc = (crc >> 1) ^ ((-((crc & 1) as i32)) as u32 & CRC_POLY);
        }
        table[0][i as usize] = crc;
    }

    for i in 0..TABLE_SIZE as u32 {
        // println!("{} crc {:#x}", i, crc);
        // / http://sourceforge.net/projects/slicing-by-8/
        // for (int slice = 1; slice < MaxSlice; slice++)
        //  Crc32Lookup[slice][i] = (Crc32Lookup[slice - 1][i] >> 8) ^
        //            Crc32Lookup[0][Crc32Lookup[slice - 1][i] & 0xFF];

        for slice in 1..SLICE_SIZE {
            let i: usize = i as usize;
            let slice_index: usize = slice - 1;
            let x: u32 = table[slice_index][i] >> 8;
            let y: u32 = table[0][(table[slice_index][i] & 0xFF) as usize];
            table[slice][i] = x ^ y;
        }
    }


    // for i in 0..MAX_SLICE {
    // println!("table {}", i);
    // for j in 0..TABLE_SIZE / 8 {
    // println!("{:#010X},{:#010X},{:#010X},{:#010X}, {:#010x},{:#010x},{:#010x},{:#010x}",
    // table[i][j * 8 + 0],
    // table[i][j * 8 + 1],
    // table[i][j * 8 + 2],
    // table[i][j * 8 + 3],
    // table[i][j * 8 + 4],
    // table[i][j * 8 + 5],
    // table[i][j * 8 + 6],
    // table[i][j * 8 + 7]);
    // }
    // }
    //
    //
    // use the table
    //
    // const uint32_t* current = (const uint32_t*) data;
    //
    // process four bytes at once (Slicing-by-4)
    // while (length >= 4)
    // {
    // uint32_t one = *current++ ^ crc;
    // crc = Crc32Lookup[0][(one>>24) & 0xFF] ^
    // Crc32Lookup[1][(one>>16) & 0xFF] ^
    // Crc32Lookup[2][(one>> 8) & 0xFF] ^
    // Crc32Lookup[3][ one      & 0xFF];
    //
    // length -= 4;
    // }
    //
    // const uint8_t* currentChar = (const uint8_t*) current;
    // remaining 1 to 3 bytes (standard algorithm)
    // while (length--)
    // crc = (crc >> 8) ^ Crc32Lookup[0][(crc & 0xFF) ^ *currentChar++];
    //
    // return ~crc; // same as crc ^ 0xFFFFFFFF
    //


    // I think it might be better to iterate over 4 bytes manually vs trying to use an iterator
    let mut crc: u32 = !0;
    let bytes = bytes;
    let mut length = bytes.len();
    for current in bytes.chunks(SLICE_SIZE) {
        // handle any bytes less than the chunksize
        if length < SLICE_SIZE {
            for &byte in current {
                crc = table[0][((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
                length -= 1;
            }
            break;
        }
        // reconstruct a u32 from bytes
        let b0: u32 = current[0] as u32;
        let b1: u32 = (current[1] as u32) << 8;
        let b2: u32 = (current[2] as u32) << 16;
        let b3: u32 = (current[3] as u32) << 24;
        let b4: u32 = current[4] as u32;
        let b5: u32 = (current[5] as u32) << 8;
        let b6: u32 = (current[6] as u32) << 16;
        let b7: u32 = (current[7] as u32) << 24;
        let b8: u32 = current[8] as u32;
        let b9: u32 = (current[9] as u32) << 8;
        let b10: u32 = (current[10] as u32) << 16;
        let b11: u32 = (current[11] as u32) << 24;
        let b12: u32 = current[12] as u32;
        let b13: u32 = (current[13] as u32) << 8;
        let b14: u32 = (current[14] as u32) << 16;
        let b15: u32 = (current[15] as u32) << 24;

        // println!("or u32 {:#x}", (b0 | b1 | b2 | b3) ^ crc);
        let one: u32 = (b0 | b1 | b2 | b3) ^ crc;
        let two: u32 = b4 | b5 | b6 | b7;
        let three: u32 = b8 | b9 | b10 | b11;
        let four: u32 = b12 | b13 | b14 | b15;
        //   ((current[0] as u32) as u32 |
        // ((current[1] as u32) << 8) as u32 |
        // ((current[2] as u32) << 16) as u32 |
        // ((current[3] as u32) << 24) as u32) ^
        // crc;
        //
        // println!("reconstructed u32 {:#x}", one);
        let xb0: u8 = (one >> 24 & 0xFF) as u8;
        let xb1: u8 = (one >> 16 & 0xFF) as u8;
        let xb2: u8 = (one >> 8 & 0xFF) as u8;
        let xb3: u8 = (one >> 0 & 0xFF) as u8;
        let xb4: u8 = (two >> 24 & 0xFF) as u8;
        let xb5: u8 = (two >> 16 & 0xFF) as u8;
        let xb6: u8 = (two >> 8 & 0xFF) as u8;
        let xb7: u8 = (two >> 0 & 0xFF) as u8;
        let xb8: u8 = (three >> 24 & 0xFF) as u8;
        let xb9: u8 = (three >> 16 & 0xFF) as u8;
        let xb10: u8 = (three >> 8 & 0xFF) as u8;
        let xb11: u8 = (three >> 0 & 0xFF) as u8;
        let xb12: u8 = (four >> 24 & 0xFF) as u8;
        let xb13: u8 = (four >> 16 & 0xFF) as u8;
        let xb14: u8 = (four >> 8 & 0xFF) as u8;
        let xb15: u8 = (four >> 0 & 0xFF) as u8;

        let tl0: u32 = table[12][xb0 as usize];
        let tl1: u32 = table[13][xb1 as usize];
        let tl2: u32 = table[14][xb2 as usize];
        let tl3: u32 = table[15][xb3 as usize];
        let tl4: u32 = table[8][xb4 as usize];
        let tl5: u32 = table[9][xb5 as usize];
        let tl6: u32 = table[10][xb6 as usize];
        let tl7: u32 = table[11][xb7 as usize];
        let tl8: u32 = table[4][xb8 as usize];
        let tl9: u32 = table[5][xb9 as usize];
        let tl10: u32 = table[6][xb10 as usize];
        let tl11: u32 = table[7][xb11 as usize];
        let tl12: u32 = table[0][xb12 as usize];
        let tl13: u32 = table[1][xb13 as usize];
        let tl14: u32 = table[2][xb14 as usize];
        let tl15: u32 = table[3][xb15 as usize];

        // let tl0: u32 = table[0][(one >> 24 & 0xFF) as usize];
        // let tl1: u32 = table[1][(one >> 16 & 0xFF) as usize];
        // let tl2: u32 = table[2][(one >> 8 & 0xFF) as usize];
        // let tl3: u32 = table[3][(one >> 0 & 0xFF) as usize];
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b0, xb0, tl0);
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b1, xb1, tl1);
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b2, xb2, tl2);
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b3, xb3, tl3);
        crc = tl0 ^ tl1 ^ tl2 ^ tl3 ^ tl4 ^ tl5 ^ tl6 ^ tl7 ^ tl8 ^ tl9 ^ tl10 ^ tl11 ^
              tl12 ^ tl13 ^ tl14 ^ tl15;

        // update how many bytes there are left to crc
        length -= SLICE_SIZE;
        // println!("crc {:#010x}", crc);
    }

    assert!(length == 0);
    // i == current
    // crc = table[0][((crc ^ current as u32) & 0xFF) as usize] ^ (crc >> 8);
    // on exit do the standard 0xFFFF_FFFF flip
    //    println!("final crc {:#010x}", !crc);
    !crc
}

// use std::io::Bytes;
// use std::iter::Iterator;

// use std::io::BufRead;
use std::io::Read;
//use itertools::Itertools;

pub fn checksum_ieee_sixteen_byte_iterator<R : Read>(mut reader: R, length: usize) -> u32 {
    const TABLE_SIZE: usize = 0x100;
    const SLICE_SIZE: usize = 16;
    let mut table: [[u32; TABLE_SIZE]; SLICE_SIZE] = [[0; TABLE_SIZE]; SLICE_SIZE];

    // build the table
    for i in 0..TABLE_SIZE as u32 {
        let mut crc: u32 = i;
        for _ in 0..8 {
            crc = (crc >> 1) ^ ((-((crc & 1) as i32)) as u32 & CRC_POLY);
        }
        table[0][i as usize] = crc;
    }

    for i in 0..TABLE_SIZE as u32 {
        // println!("{} crc {:#x}", i, crc);
        // / http://sourceforge.net/projects/slicing-by-8/
        // for (int slice = 1; slice < MaxSlice; slice++)
        //  Crc32Lookup[slice][i] = (Crc32Lookup[slice - 1][i] >> 8) ^
        //            Crc32Lookup[0][Crc32Lookup[slice - 1][i] & 0xFF];

        for slice in 1..SLICE_SIZE {
            let i: usize = i as usize;
            let slice_index: usize = slice - 1;
            let x: u32 = table[slice_index][i] >> 8;
            let y: u32 = table[0][(table[slice_index][i] & 0xFF) as usize];
            table[slice][i] = x ^ y;
        }
    } // tables set up
    
    let iters = length / 16;
    let mut current = vec![0; 16];
    let mut crc: u32 = !0;
    let mut length = length;
    for _ in 0..iters {
        _ = reader.read(&mut current[..]).unwrap();
    // I think it might be better to iterate over 4 bytes manually vs trying to use an iterator
        
    //for current in &bytes.into_iter().chunks(SLICE_SIZE) {
        // handle any bytes less than the chunksize
        // reconstruct a u32 from bytes
        let b0: u32 = current[0] as u32;
        let b1: u32 = (current[1] as u32) << 8;
        let b2: u32 = (current[2] as u32) << 16;
        let b3: u32 = (current[3] as u32) << 24;
        let b4: u32 = current[4] as u32;
        let b5: u32 = (current[5] as u32) << 8;
        let b6: u32 = (current[6] as u32) << 16;
        let b7: u32 = (current[7] as u32) << 24;
        let b8: u32 = current[8] as u32;
        let b9: u32 = (current[9] as u32) << 8;
        let b10: u32 = (current[10] as u32) << 16;
        let b11: u32 = (current[11] as u32) << 24;
        let b12: u32 = current[12] as u32;
        let b13: u32 = (current[13] as u32) << 8;
        let b14: u32 = (current[14] as u32) << 16;
        let b15: u32 = (current[15] as u32) << 24;

        // println!("or u32 {:#x}", (b0 | b1 | b2 | b3) ^ crc);
        let one: u32 = (b0 | b1 | b2 | b3) ^ crc;
        let two: u32 = b4 | b5 | b6 | b7;
        let three: u32 = b8 | b9 | b10 | b11;
        let four: u32 = b12 | b13 | b14 | b15;
        //   ((current[0] as u32) as u32 |
        // ((current[1] as u32) << 8) as u32 |
        // ((current[2] as u32) << 16) as u32 |
        // ((current[3] as u32) << 24) as u32) ^
        // crc;
        //
        // println!("reconstructed u32 {:#x}", one);
        let xb0: u8 = (one >> 24 & 0xFF) as u8;
        let xb1: u8 = (one >> 16 & 0xFF) as u8;
        let xb2: u8 = (one >> 8 & 0xFF) as u8;
        let xb3: u8 = (one >> 0 & 0xFF) as u8;
        let xb4: u8 = (two >> 24 & 0xFF) as u8;
        let xb5: u8 = (two >> 16 & 0xFF) as u8;
        let xb6: u8 = (two >> 8 & 0xFF) as u8;
        let xb7: u8 = (two >> 0 & 0xFF) as u8;
        let xb8: u8 = (three >> 24 & 0xFF) as u8;
        let xb9: u8 = (three >> 16 & 0xFF) as u8;
        let xb10: u8 = (three >> 8 & 0xFF) as u8;
        let xb11: u8 = (three >> 0 & 0xFF) as u8;
        let xb12: u8 = (four >> 24 & 0xFF) as u8;
        let xb13: u8 = (four >> 16 & 0xFF) as u8;
        let xb14: u8 = (four >> 8 & 0xFF) as u8;
        let xb15: u8 = (four >> 0 & 0xFF) as u8;

        let tl0: u32 = table[12][xb0 as usize];
        let tl1: u32 = table[13][xb1 as usize];
        let tl2: u32 = table[14][xb2 as usize];
        let tl3: u32 = table[15][xb3 as usize];
        let tl4: u32 = table[8][xb4 as usize];
        let tl5: u32 = table[9][xb5 as usize];
        let tl6: u32 = table[10][xb6 as usize];
        let tl7: u32 = table[11][xb7 as usize];
        let tl8: u32 = table[4][xb8 as usize];
        let tl9: u32 = table[5][xb9 as usize];
        let tl10: u32 = table[6][xb10 as usize];
        let tl11: u32 = table[7][xb11 as usize];
        let tl12: u32 = table[0][xb12 as usize];
        let tl13: u32 = table[1][xb13 as usize];
        let tl14: u32 = table[2][xb14 as usize];
        let tl15: u32 = table[3][xb15 as usize];

        // let tl0: u32 = table[0][(one >> 24 & 0xFF) as usize];
        // let tl1: u32 = table[1][(one >> 16 & 0xFF) as usize];
        // let tl2: u32 = table[2][(one >> 8 & 0xFF) as usize];
        // let tl3: u32 = table[3][(one >> 0 & 0xFF) as usize];
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b0, xb0, tl0);
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b1, xb1, tl1);
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b2, xb2, tl2);
        // println!("b {:#010x}, xb {:#x}, tlx {:#010x}", b3, xb3, tl3);
        crc = tl0 ^ tl1 ^ tl2 ^ tl3 ^ tl4 ^ tl5 ^ tl6 ^ tl7 ^ tl8 ^ tl9 ^ tl10 ^ tl11 ^
              tl12 ^ tl13 ^ tl14 ^ tl15;

        // update how many bytes there are left to crc
        length -= SLICE_SIZE;
        // println!("crc {:#010x}", crc);
    }
    trace!("{} length left", length);
    assert!(length < SLICE_SIZE);
    if length > 0 {
        for byte in reader.bytes() {
            let byte = byte.unwrap();
            crc = table[0][((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
            length -= 1;
            trace!("{} length left", length);
        }
    }

    assert!(length == 0);
    // i == current
    // crc = table[0][((crc ^ current as u32) & 0xFF) as usize] ^ (crc >> 8);
    // on exit do the standard 0xFFFF_FFFF flip
    //    println!("final crc {:#010x}", !crc);
    !crc
}
