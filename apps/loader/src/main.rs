#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg(feature = "axstd")]
use axstd as std;

// app running aspace
// SBI(0x80000000) -> App <- Kernel(0x80200000)
// 0xffff_ffc0_0000_0000
const PLASH_START: usize = 0x22000000;
const MAX_BLOCK: isize = 32 - 1;
const RUN_START: usize = 0xffff_ffc0_8010_0000;


struct AppManager {
    load_start: *const u8,
    // run_start: *const u8,
    current_num: usize,
    current_offset: isize,
    apps_num: usize,
    apps_offset: [isize; MAX_BLOCK as usize],
    apps_true_length: [usize; MAX_BLOCK as usize],
    apps_fake_length: [usize; MAX_BLOCK as usize],
}

impl AppManager {
    pub fn new() -> Self {
        Self {
            load_start: PLASH_START as *const u8,
            // run_start: RUN_START as *const u8,
            current_num: 0,
            current_offset: 0,
            apps_num: 0,
            apps_offset: [0; MAX_BLOCK as usize],
            apps_true_length: [0; MAX_BLOCK as usize],
            apps_fake_length: [0; MAX_BLOCK as usize],
        }
    }

    pub fn search_app(&mut self) {
        let search_length = 4;
        let mut count_length = 0;
        let mut total_apps = 0;

        while count_length < MAX_BLOCK {
            let offset = count_length;
            let load_code = get_code(self.load_start, offset, search_length);
            if !is_all_zeros(load_code) {
                total_apps += 1;
                println!("offset:{:?}", offset);
                self.apps_offset[total_apps - 1] = offset;

                self.apps_fake_length[total_apps - 1] = search_length;
                self.apps_true_length[total_apps - 1] = ensure_length(load_code).len();
            }
            count_length += 1;
        }
        println!("apps offset: {:?}", self.apps_offset);
        // println!("true length: {:?}", self.apps_true_length);
        // println!("fake length: {:?}", self.apps_fake_length);
        self.apps_num = total_apps;
    }

    // pub fn get_load_fake(&self, index: usize) -> &[u8] {
    //     get_code(self.load_start, index as isize * 4, self.apps_fake_length[index])
    // }

    pub fn get_load_true(&mut self) -> &[u8] {
        // println!("Load From AppManaer::current_num: {:?}", self.current_num);
        // println!("Load From AppManaer::current_offset: {:?}", self.current_offset);
        self.current_offset += self.apps_offset[self.current_num];
        let code = ensure_length(get_code(
            self.load_start,
            self.current_offset,
            self.apps_fake_length[self.current_num],
        ));
        // println!("Load From AppManaer::next_step: {:?}", self.current_offset);
        self.current_forward();
        code
    }
    pub fn get_true_code(&self, index: usize) -> &[u8] {
        let code = ensure_length(get_code(
            self.load_start,
            self.current_offset,
            self.apps_fake_length[index],
        ));
        code
        //     // println!("Load From AppManaer::next_step: {:?}", self.current_offset);
    }

    pub fn current_forward(&mut self) {
        self.current_num += 1;
    }
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let mut app_manager = AppManager::new();
    app_manager.search_app();
    let apps_num = app_manager.apps_num;
    let mut run_start = RUN_START;
    for _ in 0..apps_num {
        let load_code = app_manager.get_load_true();
        println!("load code: {:?}", load_code);
        let run_code =
            unsafe { core::slice::from_raw_parts_mut(run_start as *mut u8, load_code.len()) };
        run_code.fill(0);        
        run_start += load_code.len();
        run_code.copy_from_slice(load_code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());
    
    }
    println!("Load Done");
    println!("Exe App...");
    unsafe {
        core::arch::asm!("
        li      t2, {start}
        jalr    t2
        ",
            start = const RUN_START,
        )
    }
    unsafe {
        core::arch::asm!("
        li      t2, {run_start}
        jalr    t2",
        run_start = const RUN_START+2,
        )
    }

}

fn ensure_length(code: &[u8]) -> &[u8] {
    if code.len() == 4 && &code[2..] == [0, 0] {
        &code[..2]
    } else {
        code
    }
}

fn is_all_zeros(slice: &[u8]) -> bool {
    slice.iter().all(|&x| x == 0)
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
