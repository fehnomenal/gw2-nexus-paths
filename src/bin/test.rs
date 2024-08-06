use windows::Win32::{Foundation::FreeLibrary, System::LibraryLoader::LoadLibraryA};
use windows_strings::s;

fn main() {
    let lib = unsafe { LoadLibraryA(s!("paths.dll")) }.unwrap();

    dbg!(lib);

    unsafe { FreeLibrary(lib) }.unwrap();
}
