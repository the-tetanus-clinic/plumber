extern crate libc;
use self::libc::{c_char, c_int};
use std::ffi::{CString, CStr};
use std::mem;
use std::str::from_utf8;

#[derive(PartialEq,Debug,Clone)]
pub enum Type {
	// valid dnsRR_Header.Rrtype and dnsQuestion.qtype
	A     = 1,
	NS    = 2,
	MD    = 3,
	MF    = 4,
	CNAME = 5,
	SOA   = 6,
	MB    = 7,
	MG    = 8,
	MR    = 9,
	NULL  = 10,
	WKS   = 11,
	PTR   = 12,
	HINFO = 13,
	MINFO = 14,
	MX    = 15,
	TXT   = 16,
	AAAA  = 28,
	SRV   = 33,

	// valid dnsQuestion.qtype only
	AXFR  = 252,
	MAILB = 253,
	MAILA = 254,
	ALL   = 255,
}

#[derive(PartialEq,Debug,Clone)]
pub enum Class {
	// valid dnsQuestion.qclass
	INET   = 1,
	CSNET  = 2,
	CHAOS  = 3,
	HESIOD = 4,
	ANY    = 255,
}

#[derive(PartialEq,Debug,Clone)]
pub enum Rcode {
	// dnsMsg.rcode
	Success        = 0,
	FormatError    = 1,
	ServerFailure  = 2,
	NameError      = 3,
	NotImplemented = 4,
	Refused        = 5,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum ns_sect_q {
    ns_s_qd = 0,        /* Query: Question. */
    ns_s_an = 1,        /* Query: Answer. */
    ns_s_ns = 2,        /* Query: Name servers. */
    ns_s_ar = 3,        /* Query|Update: Additional records. */
    ns_s_max = 4
}

#[repr(C)]
pub struct ns_rr {
    name: [u8;1025],
    typef: u16,
    rr_class: u16,
    ttl: u32,
    rdlength: u16,
    rdata: *const u8,
}

impl Default for ns_rr {
    fn default() -> ns_rr {
        ns_rr {
            name: [0u8;1025],
            typef: 0,
            rr_class: 0,
            ttl: 0,
            rdlength: 0,
            rdata: 0 as *const u8,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ns_msg {
    msg: *const u8,
    eom: *const u8,
    id: u16,
    flags: u16,
    counts: [u16;4],
    sections: [*const u8;4],
    sect: ns_sect_q,
    rrnum: c_int,
    msg_ptr: *const u8,
}

impl Default for ns_msg {
    fn default() -> ns_msg {
        ns_msg {
            msg: 0 as *const u8,
            eom: 0 as *const u8,
            id: 0,
            flags: 0,
            counts: [0,0,0,0],
            sections: [0 as *const u8, 0 as *const u8, 0 as *const u8, 0 as *const u8],
            sect: ns_sect_q::ns_s_qd,
            rrnum: 0,
            msg_ptr: 0 as *const u8,
        }
    }
}

#[link(name = "resolv")]
extern {
    pub fn __res_query(dname: *const c_char, class: c_int, typef: c_int,
               answer: *const u8, anslen: c_int) -> c_int;
    pub fn ns_initparse(answer: *const u8, len: c_int, dst: *mut ns_msg);
    pub fn ns_parserr(msg: *mut ns_msg, sect: ns_sect_q, which: c_int, rr: *mut ns_rr);
    pub fn ns_sprintrr(msg: *mut ns_msg, rr: *mut ns_rr, b1: *const c_char,
                       b2: *const c_char, buf: *const c_char, buflen: c_int);
}

fn query(name: &str, class: Class, typef: Type) -> Result<(u16, [u8;4]), Rcode> {
    let ns_c_in = 1;
    let ns_t_srv = 33;
    let dname = CString::new(name).unwrap();
    let ans_buf = [0u8;4096];
    let mut msg = ns_msg{..Default::default() };
    unsafe {
        println!("before");
        let len = __res_query(dname.as_ptr() as *const i8, class as i32, typef as i32,
                           &ans_buf as *const u8, 4096);
        ns_initparse(&ans_buf as *const u8, len, &mut msg as *mut ns_msg);
        let nmsg = msg.counts[1] as c_int;
        for i in 0..nmsg {
            let dispbuf = [0u8;4096];
            let mut rr = ns_rr{..Default::default() };
            ns_parserr(&mut msg as *mut ns_msg, ns_sect_q::ns_s_an, i, &mut rr as *mut ns_rr);
            ns_sprintrr(&mut msg as *mut ns_msg, &mut rr as *mut ns_rr,
                        0 as *const c_char, 0 as *const c_char,
                        dispbuf.as_ptr() as *const i8, 4096);
            let c_str = unsafe { CStr::from_ptr(dispbuf.as_ptr() as *const i8) };
            let s = from_utf8(c_str.to_bytes()).unwrap().to_owned();
            println!("dispbuf: {}", s);
        }

    }
    Err(Rcode::NotImplemented)
}

#[test]
fn test_query() {
    let r = query("_etcd-server._tcp.etcd-t1.mesos", Class::INET, Type::SRV);
    println!("result: {:?}", r);
}
