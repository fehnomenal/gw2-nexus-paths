#[no_mangle]
#[allow(non_snake_case)]
pub fn DllMain(hInstDll: *const u8, fdwReason: u32, lpvReserved: *const u8) -> u32 {
    dbg!(hInstDll, fdwReason, lpvReserved);

    1
}

#[no_mangle]
pub fn add(left: isize, right: isize) -> isize {
    left + right
}
