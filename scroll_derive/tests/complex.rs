use scroll::Pread;
use scroll_derive::{Pread, Pwrite};

#[derive(Debug, Pread, Pwrite)]
pub struct FloofHeader {
    pub boop: u8,
    pub length: u16,
    pub flag: u8,
    pub quux_service: u8,
    pub quux_client_id: u8,
}

pub trait QuuxHeaderApi {
    fn msg_len(&self) -> usize;
}

#[derive(Debug, Pread, Pwrite)]
pub struct QuuxHeader {
    pub typ: u8,
    pub txn_id: u16,
    pub msg_id: u16,
    pub msg_len: u16,
}

impl QuuxHeaderApi for QuuxHeader {
    fn msg_len(&self) -> usize {
        self.msg_len as usize
    }
}

#[derive(Debug, Pread, Pwrite)]
pub struct Response<'a, T: QuuxHeaderApi> {
    pub floof: FloofHeader,
    pub quux: T,
    #[scroll(ctx = quux.msg_len())]
    pub tlvs: &'a [u8],
}

#[test]
fn test_pread_generics_with_ctx_dependent_on_prior_field() {
    let bytes = [
        1, 70, 0, 128, 9, 3, 4, 1, 0, 46, 0, 58, 0, 1, 8, 0, 1, 1, 4, 0, 1, 6, 0, 0, 16, 16, 0, 1,
        1, 0, 12, 43, 47, 52, 48, 49, 57, 42, 54, 49, 53, 50, 51, 39, 3, 0, 1, 1, 2, 43, 19, 0, 1,
        1, 0, 0, 0, 1, 12, 41, 49, 51, 48, 49, 57, 42, 54, 43, 51, 50, 54,
    ];
    let resp: Response<QuuxHeader> = bytes.pread(0).unwrap();
    assert_eq!(resp.floof.flag, 0x80);
    assert_eq!(resp.floof.length as usize, bytes.len() - 1);
    assert_eq!(resp.floof.quux_service, 0x9);
    assert_eq!(resp.floof.quux_client_id, 3);
    assert_eq!(resp.quux.msg_len(), resp.tlvs.len());
    assert_eq!(resp.quux.typ, 0x4);
    assert_eq!(resp.quux.txn_id, 1);
}
