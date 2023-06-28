use std::any::Any;
use cosmwasm_std::{Binary, ContractResult, SystemError, SystemResult};
use cosmwasm_vm::{BackendResult, GasInfo, Querier};

use crate::error::GoError;
use crate::memory::{U8SliceView, UnmanagedVector};
use std::os::raw::{c_char, c_void};
use crate::db::Db;
use std::ffi::{CStr, CString};

// this represents something passed in from the caller side of FFI
#[repr(C)]
#[derive(Clone)]
pub struct querier_t {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone)]
pub struct Querier_vtable {
    // We return errors through the return buffer, but may return non-zero error codes on panic
    pub query_external: extern "C" fn(
        *const querier_t,
        u64,
        *mut u64,
        U8SliceView,
        *mut UnmanagedVector, // result output
        *mut UnmanagedVector, // error message output
    ) -> i32,
    pub generate_call_info: extern "C" fn (
        *const querier_t,
        *mut c_char,
        *mut *mut c_char,
        *mut Db,
        *mut GoQuerier,
    ) -> i32,
}

#[repr(C)]
#[derive(Clone)]
pub struct GoQuerier {
    pub state: *const querier_t,
    pub vtable: Querier_vtable,
}

// TODO: check if we can do this safer...
unsafe impl Send for GoQuerier {}

impl Querier for GoQuerier {
    fn query_raw(
        &self,
        request: &[u8],
        gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_result: GoError = (self.vtable.query_external)(
            self.state,
            gas_limit,
            &mut used_gas as *mut u64,
            U8SliceView::new(Some(request)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

        let gas_info = GasInfo::with_externally_used(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || {
            format!(
                "Failed to query another contract with this request: {}",
                String::from_utf8_lossy(request)
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        let bin_result: Vec<u8> = output.unwrap_or_default();
        let result = serde_json::from_slice(&bin_result).or_else(|e| {
            Ok(SystemResult::Err(SystemError::InvalidResponse {
                error: format!("Parsing Go response: {}", e),
                response: bin_result.into(),
            }))
        });
        (result, gas_info)
    }

    fn generate_call_info(&self, contract_address: String) -> [u8; 32] {
        println!("wasmvm generate_call_info contract_address: {:?}", contract_address);
        let c_string = CString::new(contract_address).expect("Failed to create CString");
        let c_string_ptr = c_string.into_raw() as *mut c_char;

        let mut res_code_hash: *mut c_char = std::ptr::null_mut();
        let mut res_store: *mut Db = std::ptr::null_mut();
        let mut res_querier: *mut GoQuerier = std::ptr::null_mut();

        let go_result: GoError = (self.vtable.generate_call_info)(
            self.state,
            c_string_ptr,
            &mut res_code_hash,
            *&mut res_store,
            *&mut res_querier,
        ).into();

        // 释放c_string_ptr
        unsafe {
            let _ = CString::from_raw(c_string_ptr);
        }

        let mut byte_array: [u8; 32] = [0; 32];
        if !res_code_hash.is_null(){
            let c_str = unsafe { CStr::from_ptr(res_code_hash) };
            let byte_slice = c_str.to_bytes();
            byte_array.copy_from_slice(&byte_slice[..32]);
            return byte_array
        }


        if !res_code_hash.is_null() && !res_store.is_null() && !res_querier.is_null() {
            let c_str = unsafe { CStr::from_ptr(res_code_hash) };
            let byte_slice = c_str.to_bytes();
            byte_array.copy_from_slice(&byte_slice[..32]);
            //
            let res_store_owned = unsafe { Box::from_raw(res_store) };

            // TODO 释放资源
            // unsafe {
            //     // 释放 res_code_hash
            //     if !res_code_hash.is_null() {
            //         libc::free(res_code_hash as *mut libc::c_void);
            //     }
            //
            //     // 释放 res_store
            //     if !res_store.is_null() {
            //         libc::free(res_store);
            //     }
            //
            //     // 释放 res_querier
            //     if !res_querier.is_null() {
            //         libc::free(res_querier);
            //     }
            //
            //     // 释放 res_gas_meter
            //     if !res_gas_meter.is_null() {
            //         libc::free(res_gas_meter);
            //     }
            // }
            return byte_array
        } else {
            // TODO the else
            return byte_array
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
