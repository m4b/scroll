#![feature(test)]
extern crate test;
extern crate byteorder;
extern crate scroll;
extern crate rayon;
//extern crate byteio;

use test::black_box;

#[bench]
fn bench_parallel_cread_with(b: &mut test::Bencher) {
    use scroll::{Cread, LE};
    use rayon::prelude::*;
    let vec = vec![0u8; 1_000_000];
    let nums = vec![0usize; 500_000];
    b.iter(|| {
        let data = black_box(&vec[..]);
        nums.par_iter().for_each(| offset | {
            let _: u16 = black_box(data.cread_with(*offset, LE));
        });
    });
    b.bytes = vec.len() as u64;
}

#[bench]
fn bench_cread_vec(b: &mut test::Bencher) {
    use scroll::{Cread, LE};
    let vec = vec![0u8; 1_000_000];
    b.iter(|| {
        let data = black_box(&vec[..]);
        for val in data.chunks(2) {
            let _: u16 = black_box(val.cread_with(0, LE));
        }
    });
    b.bytes = vec.len() as u64;
}

#[bench]
fn bench_cread(b: &mut test::Bencher) {
    use scroll::{Cread};
    const NITER: i32 = 100_000;
    b.iter(|| {
        for _ in 1..NITER {
            let data = black_box([1, 2]);
            let _: u16 = black_box(data.cread(0));
        }
    });
    b.bytes = 2 * NITER as u64;
}

#[bench]
fn bench_pread_ctx_vec(b: &mut test::Bencher) {
    use scroll::{Pread};
    let vec = vec![0u8; 1_000_000];
    b.iter(|| {
        let data = black_box(&vec[..]);
        for val in data.chunks(2) {
            let _: Result<u16, _> = black_box(val.pread(0));
        }
    });
    b.bytes = vec.len() as u64;
}

#[bench]
fn bench_pread_with_unwrap(b: &mut test::Bencher) {
    use scroll::{Pread, LE};
    const NITER: i32 = 100_000;
    b.iter(|| {
        for _ in 1..NITER {
            let data: &[u8] = &black_box([1, 2]);
            let _: u16 = black_box(data.pread_with(0, LE).unwrap());
        }
    });
    b.bytes = 2 * NITER as u64;
}

#[bench]
fn bench_pread_vec(b: &mut test::Bencher) {
    use scroll::{Pread, LE};
    let vec = vec![0u8; 1_000_000];
    b.iter(|| {
        let data = black_box(&vec[..]);
        for val in data.chunks(2) {
            let _: Result<u16, _> = black_box(val.pread_with(0, LE));
        }
    });
    b.bytes = vec.len() as u64;
}

#[bench]
fn bench_pread_unwrap(b: &mut test::Bencher) {
    use scroll::{Pread};
    const NITER: i32 = 100_000;
    b.iter(|| {
        for _ in 1..NITER {
            let data = black_box([1, 2]);
            let _: u16 = black_box(data.pread(0)).unwrap();
        }
    });
    b.bytes = 2 * NITER as u64;
}

#[bench]
fn bench_pread_unsafe(b: &mut test::Bencher) {
    use scroll::{Pread, LE};
    const NITER: i32 = 100_000;
    b.iter(|| {
        for _ in 1..NITER {
            let data = black_box([1, 2]);
            let _: u16 = black_box(data.pread_unsafe(0, LE));
        }
    });
    b.bytes = 2 * NITER as u64;
}

#[bench]
fn bench_gread_vec(b: &mut test::Bencher) {
    use scroll::{Gread};
    let vec = vec![0u8; 1_000_000];
    b.iter(|| {
        let data = black_box(&vec[..]);
        for val in data.chunks(2) {
            let mut offset = 0;
            let _: Result<u16, _> = black_box(val.gread(&mut offset));
        }
    });
    b.bytes = vec.len() as u64;
}

#[bench]
fn bench_gread_unsafe(b: &mut test::Bencher) {
    use scroll::{Gread, LE};
    const NITER: i32 = 100_000;
    b.iter(|| {
        for _ in 1..NITER {
            let data = black_box([1, 2]);
            let mut offset = 0;
            let _: u16 = black_box(data.gread_unsafe(&mut offset, LE));
        }
    });
    b.bytes = 2 * NITER as u64;
}

#[bench]
fn bench_parallel_pread_with(b: &mut test::Bencher) {
    use scroll::{Pread, LE};
    use rayon::prelude::*;
    let vec = vec![0u8; 1_000_000];
    let nums = vec![0usize; 500_000];
    b.iter(|| {
        let data = black_box(&vec[..]);
        nums.par_iter().for_each(| offset | {
            let _: Result<u16, _> = black_box(data.pread_with(*offset, LE));
        });
    });
    b.bytes = vec.len() as u64;
}

#[bench]
fn bench_byteorder_vec(b: &mut test::Bencher) {
    use byteorder::ReadBytesExt;
    let vec = vec![0u8; 1_000_000];
    b.iter(|| {
        let data = black_box(&vec[..]);
        for mut val in data.chunks(2) {
            let _: Result<u16, _> = black_box(val.read_u16::<byteorder::LittleEndian>());
        }
    });
    b.bytes = vec.len() as u64;
}

#[bench]
fn bench_byteorder(b: &mut test::Bencher) {
    use byteorder::ByteOrder;
    const NITER: i32 = 100_000;
    b.iter(|| {
        for _ in 1..NITER {
            let data = black_box([1, 2]);
            let _: u16 = black_box(byteorder::LittleEndian::read_u16(&data));
        }
    });
    b.bytes = 2 * NITER as u64;
}

// #[bench]
// fn bench_byteio_vec(b: &mut test::Bencher) {
//     use byteio::ReadBytesExt;
//     let vec = vec![0u8; 1_000_000];
//     b.iter(|| {
//         let data = black_box(&vec[..]);
//         for mut val in data.chunks(2) {
//             let _: Result<u16, _> = black_box(val.read_as::<byteio::LittleEndian>());
//         }
//     });
//     b.bytes = vec.len() as u64;
// }

// #[bench]
// fn bench_byteio(b: &mut test::Bencher) {
//     use byteio::ByteOrder;
//     const NITER: i32 = 100_000;
//     b.iter(|| {
//         for _ in 1..NITER {
//             let data = black_box([1, 2]);
//             let _: u16 = black_box(byteio::LittleEndian::from_bytes(data));
//         }
//     });
//     b.bytes = 2 * NITER as u64;
// }
