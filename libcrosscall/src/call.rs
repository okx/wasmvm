use std::ffi::{CStr, CString};
use cosmwasm_vm::{Querier, Storage};
use std::os::raw::{c_char, c_void};
use crate::ffi::GenerateCallerInfo;
use wasmvm::{Db, GoQuerier};



pub fn generate_call_info<S: Storage, Q: Querier>(inquer : &dyn Querier, contract_address: String) -> [u8; 32] {
    match inquer.downcast_ref::<GoQuerier>() {
        Some(quer) => {
           // let quer: &GoQuerier = inquer.downcast::<GoQuerier>().unwrap();
            let state_ptr = unsafe {
                (*quer).state
            };

            let converted_ptr = state_ptr as *mut c_void;

            let c_string = CString::new(contract_address).expect("Failed to create CString");
            let c_string_ptr = c_string.into_raw() as *mut c_char;

            // 传入空指针便于c将结果填入
            let mut res_code_hash: *mut c_char = std::ptr::null_mut();
            // let mut res_store: Db = Db { /* 根据实际情况初始化字段 */ };
            // let mut res_querier: GoQuerier = GoQuerier { /* 根据实际情况初始化字段 */ };
            let mut res_store: *mut Db = std::ptr::null_mut();
            let mut res_querier: *mut GoQuerier = std::ptr::null_mut();
            unsafe {
                GenerateCallerInfo(
                    converted_ptr,
                    c_string_ptr,
                    &mut res_code_hash,
                    *&mut res_store,
                    *&mut res_querier,
                );

                // 释放c_string_ptr
                let _ = CString::from_raw(c_string_ptr);
            }

            let mut byte_array: [u8; 32] = [0; 32];


            if !res_code_hash.is_null() && !res_store.is_null() && !res_querier.is_null() {
                let c_str = unsafe { CStr::from_ptr(res_code_hash) };
                let byte_slice = c_str.to_bytes();
                byte_array.copy_from_slice(&byte_slice[..32]);

                //
                let res_store_owned = unsafe { Box::from_raw(res_store) };

                return byte_array
            } else {

            }
        }
        // 处理其他未知类型
        _ => {
            println!("Unknown animal type");
        }
    }
    let mut byte_array: [u8; 32] = [0; 32];
    return byte_array;
}
