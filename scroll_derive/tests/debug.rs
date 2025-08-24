use scroll_derive::*;

// These are repetitive tests but they're also problematic ones when derive macro fails
// it's easier to test/debug macro by commenting all but one that's failing and then running:
// RUSTFLAGS=-Zmacro-backtrace cargo +nightly expand --test debug
// and then investigating from there what to fix in the macro itself

#[derive(Pread, Pwrite)]
#[repr(C)]
struct Data8<T, Y> {
    ids: [T; 3],
    xyz: Y,
}

#[derive(Debug, PartialEq, Eq, Pread, Pwrite, IOread, IOwrite, SizeWith)]
struct Data10I(u8, u16);

#[derive(Debug, Pread, Pwrite)]
struct Life1<'b> {
    #[scroll(ctx = scroll::ctx::StrCtx::Length(6))]
    ids: &'b str,
    #[scroll(ctx = 5)]
    data: &'b [u8],
}

#[derive(Pread, Pwrite, SizeWith, IOread, IOwrite)]
struct TestHygenic {
    ctx: u8,
    offset: u32,
    src: i8,
    __offset: u8,
    _offset_: u8,
    _src: u8,
    _offset: u8,
}

#[derive(Pread, Pwrite)]
struct TestHygenicCtx<'a> {
    ctx: u8,
    #[scroll(ctx = ctx as usize)]
    _ctx: &'a [u8],
}
