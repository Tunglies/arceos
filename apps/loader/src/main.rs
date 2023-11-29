#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

use core::num;

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0x22000000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
        let apps_start = PLASH_START as *const u8;
        let (num, index) = find_app();
        
        let mut offset = 0;
        for i in 0..num {
            let len: usize = index[i as usize] as usize;
            let code = get_code(apps_start, offset, len);
        println!("APP Number: {:?}", i+1);
        println!("APP Length: {:?}", len);
        println!("APP Content: {:?}", code);
        offset += len as isize;
    }
}

const MAX_BLOCK: isize = 32 - 1;
fn find_app()  -> (isize, [isize; MAX_BLOCK as usize]){
    let apps_start = PLASH_START as *const u8;
    let len = 4;
    let mut count: isize = 0;
    let mut num = 0;
    let mut index = [0; MAX_BLOCK as usize];
    while count < MAX_BLOCK {
        let offset = count;
        let code = get_code(apps_start, offset, len);
        if is_not_all_zeros(&code[0..len]) {
            // println!("not all zeros, offset: {:?} count: {:?}, num: {:?}", offset, count, num);
            if count != 0 {
                index[num-1] = count;
            }
            num += 1;
        }
        count += 1;

    }
    // println!("ensure: {:?}", ensure_index(index));
    (num as isize, ensure_index(index))
}

fn ensure_index(mut index: [isize; MAX_BLOCK as usize]) -> [isize; MAX_BLOCK as usize]
{
    for item in index.iter_mut() {
        if *item == 0 {
            *item = 4;
            break;
        }
    }
    index
}

fn is_not_all_zeros(slice: &[u8]) -> bool {
    !slice.iter().all(|&x| x == 0)
}

fn get_code(apps_start: *const u8, offset: isize, len: usize) -> &'static [u8] {
    unsafe { core::slice::from_raw_parts(apps_start.offset(offset * 1024 * 1024), len) }
}

// #[inline]
// fn bytes_to_usize(bytes: &[u8]) -> usize {
//     if bytes.len() == 8 {
//         usize::from_be_bytes(bytes.try_into().unwrap())
//     } else {
//         let mut result_bytes = [0; 8];
//         result_bytes[..bytes.len()].copy_from_slice(&bytes);
//         usize::from_be_bytes(result_bytes)
//     }
// }