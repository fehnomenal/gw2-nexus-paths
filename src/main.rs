use windows::Win32::System::LibraryLoader::LoadLibraryA;
use windows_strings::s;

fn main() {
    let lib = unsafe { LoadLibraryA(s!("libpaths.dll")) }.unwrap();

    dbg!(lib);
}