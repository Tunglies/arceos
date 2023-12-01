#![feature(asm_const)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#[cfg_attr(feature = "axstd", no_mangle)]

#[cfg(feature = "axstd")]
use axstd::println;
#[cfg(feature = "axstd")]
use axstd::process;


const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];

const PLASH_START: usize = 0x22000000;
const MAX_BLOCK: isize = 32 - 1;
const RUN_START: usize = 0xffff_ffc0_8010_0000;

fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}

fn abi_shutdown() {
    println!("[ABI:Shutdown]");
    process::exit(0);
}

struct AppManager {
    load_start: *const u8,
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
            current_num: 0,
            current_offset: 0,
            apps_num: 0,
            apps_offset: [0; MAX_BLOCK as usize],
            apps_true_length: [0; MAX_BLOCK as usize],
            apps_fake_length: [0; MAX_BLOCK as usize],
        }
    }

    pub fn search_app(&mut self) {
        let search_length = 1024;
        let mut count_length = 0;
        let mut total_apps = 0;

        while count_length < MAX_BLOCK {
            let offset = count_length;
            let load_code = get_code(self.load_start, offset, search_length);
            if !is_all_zeros(load_code) {
                total_apps += 1;
                self.apps_offset[total_apps - 1] = offset;
                self.apps_fake_length[total_apps - 1] = search_length;
                self.apps_true_length[total_apps - 1] = ensure_length(load_code).len();
            } else {
                break
            }
            count_length += 1;
        }
        self.apps_num = total_apps;
    }

    pub fn get_load_true(&mut self) -> &[u8] {
        self.current_offset += self.apps_offset[self.current_num];
        let code = ensure_length(get_code(
            self.load_start,
            self.current_offset,
            self.apps_fake_length[self.current_num],
        ));
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
    }

    pub fn current_forward(&mut self) {
        self.current_num += 1;
    }
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_TERMINATE, abi_shutdown as usize);

    let mut app_manager = AppManager::new();
    app_manager.search_app();
    let apps_num = app_manager.apps_num;
    for _ in 0..apps_num {
        let load_code = app_manager.get_load_true();
        println!("load code: {:?}", load_code);
        let run_code =
            unsafe { core::slice::from_raw_parts_mut(RUN_START as *mut u8, load_code.len()) };
        run_code.fill(0);        
        run_code.copy_from_slice(load_code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        unsafe { core::arch::asm!("
            la      a7, {abi_table}
            li      t2, {run_start}
            jalr    t2
            ",
            run_start = const RUN_START,
            abi_table = sym ABI_TABLE,
        )}
    }
}

fn get_code(apps_start: *const u8, offset: isize, len: usize) -> &'static [u8] {
    unsafe { core::slice::from_raw_parts(apps_start.offset(offset * 1024 * 1024), len) }
}

fn ensure_length(code: &[u8]) -> &[u8] {
    if code.len() == 4 && &code[2..] == [0, 0] {
        &code[..2]
    } else {
        if let Some(index) = code.windows(8).position(|window| window == &[0, 0, 0, 0, 0, 0, 0, 0]) {
            &code[..index]
        } else {
            code
        }
    }
}

fn is_all_zeros(slice: &[u8]) -> bool {
    slice.iter().all(|&x| x == 0)
}

#[cfg_attr(feature = "axstd", no_mangle)]

#[inline]
fn bytes_to_usize(bytes: &[u8]) -> usize {
    if bytes.len() == 8 {
        usize::from_be_bytes(bytes.try_into().unwrap())
    } else {
        let mut result_bytes = [0; 8];
        result_bytes[..bytes.len()].copy_from_slice(&bytes);
        usize::from_be_bytes(result_bytes)
    }
}
