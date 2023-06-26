#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::os::raw::{c_char, c_void};
use wasmvm::{Db, GoQuerier};

#[link(name = "GenerateCallerInfo")]
extern "C" {
    pub fn GenerateCallerInfo(
        p: *mut c_void,
        contractAddress: *mut c_char,
        resCodeHash: *mut *mut c_char,
        resStore: *mut Db,
        resQuerier: *mut GoQuerier,
    );
}

