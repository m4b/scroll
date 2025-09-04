use scroll::Pread;
use scroll_derive::{Pread, Pwrite};

const GOOD: u16 = 0x10;
const BAD: u16 = 0x20;
const UGLY: u16 = 0x30;

#[derive(Debug, PartialEq, Pread, Pwrite)]
#[repr(u16)]
enum Service {
    Good = GOOD,
    Bad = BAD,
    Ugly = UGLY,
}

fn main() {
    let bytes = [0x10, 0x0, 0x30, 0x0, 0x20, 0x0];
    fn services(bytes: &[u8]) -> impl Iterator<Item = Service> {
        bytes
            .chunks(std::mem::size_of::<Service>())
            .filter_map(|chunk| chunk.pread(0).ok())
    }
    let services = services(&bytes);
    use Service::*;
    for (s1, s2) in services.zip([Good, Ugly, Bad].into_iter()) {
        println!("{s1:?},{s2:?}");
        assert_eq!(s1, s2);
    }
}
