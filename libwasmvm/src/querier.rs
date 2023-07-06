use std::any::Any;
use std::borrow::Borrow;
use cosmwasm_std::{Binary, ContractResult, SystemError, SystemResult};
use cosmwasm_vm::{BackendResult, GasInfo, Querier, Storage};
use cosmwasm_std::{Order, Record};

use crate::error::GoError;
use crate::memory::{U8SliceView, UnmanagedVector};
use std::os::raw::{c_char, c_void};
use crate::db::Db;
use std::ffi::{CStr, CString};
use crate::storage::GoStorage;
use crate::api::GoApi;
use crate::cache::{cache_t, to_cache};
use std::panic;
use std::env;

use cosmwasm_vm::{call_execute, call_execute_raw, Backend, Cache, Checksum, Instance, Environment, BackendApi};
use cosmwasm_vm::{process_gas_info, write_to_contract, VmResult, InstanceOptions, InternalCallParam, CacheOptions,
                  features_from_csv,Size};
use cosmwasm_std::{WasmMsg, MessageInfo, Coin, to_vec, Env, Empty};
// use tempfile::TempDir;
use std::path::PathBuf;
use std::rc::Rc;

use crate::calls::{get_global_cache};

// this represents something passed in from the caller side of FFI
#[repr(C)]
#[derive(Clone)]
pub struct querier_t {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone)]
#[derive(Debug)]
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
        *mut UnmanagedVector,
        *mut *mut Db,
        *mut *mut GoQuerier,
    ) -> i32,
    pub get_wasm_info: extern "C" fn (
        *mut *mut GoApi,
        *mut *mut cache_t,
    ) -> i32,
}

#[repr(C)]
#[derive(Clone)]
#[derive(Debug)]
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

    fn call<A: BackendApi + 'static, S: Storage, Q: Querier>(&self, env1: &Environment<A, S, Q>,
                          contract_address: String,
                          info: &MessageInfo,
                          call_msg: &[u8],
                          block_env: &Env,
                          gas_limit: u64
    ) -> (VmResult<Vec<u8>>, GasInfo) {
        env::set_var("RUST_BACKTRACE", "full");

        println!("wasmvm call contract_address: {:?} , sender address {:?}", contract_address, info.sender.clone());
        let c_string = CString::new(contract_address).expect("Failed to create CString");
        let c_string_ptr = c_string.into_raw() as *mut c_char;

        let mut res_code_hash = UnmanagedVector::default();
        let mut res_store: *mut Db = std::ptr::null_mut();
        let mut res_querier: *mut GoQuerier = std::ptr::null_mut();

        let go_result: GoError = (self.vtable.generate_call_info)(
            self.state,
            c_string_ptr,
            &mut res_code_hash as *mut UnmanagedVector,
            &mut res_store,
            &mut res_querier,
        ).into();

        // 释放c_string_ptr
        unsafe {
            let _ = CString::from_raw(c_string_ptr);
        }

        if res_store.is_null() {
            println!("wasmvm generate_call_info res_store is null");
        }

        if res_querier.is_null() {
            println!("wasmvm generate_call_info res_querier is null");
        }


        // We destruct the UnmanagedVector here, no matter if we need the data.
        let res_code_hash = res_code_hash.consume();
        let bin_res_code_hash: Vec<u8> = res_code_hash.unwrap_or_default();
        let mut byte_array: [u8; 32] = [0; 32];
        byte_array.copy_from_slice(&bin_res_code_hash[..32]);
        let querier = unsafe{(*res_querier).clone()};
        let storage = GoStorage::new(unsafe{(*res_store).clone()});




        let mut res_api: *mut GoApi = std::ptr::null_mut();
        let mut res_cache: *mut cache_t = std::ptr::null_mut();

        let go_result: GoError = (self.vtable.get_wasm_info)(
            &mut res_api,
            &mut res_cache,
        ).into();
        if res_api.is_null() {
            println!("wasmvm get_wasm_info call res_api is null");
        }
        if res_cache.is_null() {
            println!("wasmvm get_wasm_info call res_cache is null");
        }
        let api = unsafe{(*res_api).clone()};

        do_call(env1, block_env, storage, querier, api, res_cache, info, call_msg, byte_array, gas_limit)

        // if !res_code_hash.is_null(){
            //let c_str = unsafe { CStr::from_ptr(res_code_hash) };
            //let byte_slice = c_str.to_bytes();
            //byte_array.copy_from_slice(&byte_slice[..32]);
            // let querier_box: Box<dyn Querier> = Box::new(unsafe { *Box::from_raw(res_querier) });
            // let storage_box: Box<dyn Storage> = Box::new(GoStorage::new(unsafe { *Box::from_raw(res_store) }));

            //let querier_box: Box<dyn Querier> = unsafe { Box::new((*res_querier)) };
            //let storage_box: Box<dyn Storage> = Box::new(GoStorage::new(unsafe { *Box::from_raw(res_store) }));

            // let querier_box = Box::new(unsafe{(*res_querier).clone()});
            // let storage_box = Box::new(GoStorage::new(unsafe { (*res_store).clone() }));
            // //let querier_box = Box::new(GoQuerier1::default());
            // //let storage_box = Box::new(GoStorage1::default());
            // return (byte_array, querier_box, storage_box);
        // }

        // let mut byte_array: [u8; 32] = [0; 32];
        // if !res_code_hash.is_null(){
        //     let c_str = unsafe { CStr::from_ptr(res_code_hash) };
        //     let byte_slice = c_str.to_bytes();
        //     byte_array.copy_from_slice(&byte_slice[..32]);
        // }
        //
        // let storage_box = Box::new(GoStorage1::default());
        // //let querier_box = Box::new(GoQuerier1::default());
        // let mut  querier_box: Box<dyn Querier>;
        // let result = panic::catch_unwind(|| {
        //     querier_box = unsafe { Box::from_raw(res_querier) };
        // });
        //
        // match result {
        //     Ok(_) => {
        //         println!("Panic not occurred");
        //         return (byte_array, querier_box, storage_box);
        //     }
        //     Err(_) => {
        //         println!("Panic occurred and caught");
        //         querier_box = Box::new(GoQuerier1::default());
        //         return (byte_array, querier_box, storage_box);
        //     }
        // }

        // //if !res_code_hash.is_null() && !res_store.is_null() && !res_querier.is_null() {
        //     let c_str = unsafe { CStr::from_ptr(res_code_hash) };
        //     let byte_slice = c_str.to_bytes();
        //     byte_array.copy_from_slice(&byte_slice[..32]);
        //
        //     //let querier_box: Box<dyn Querier> = Box::new(unsafe { *Box::from_raw(res_querier) });
        //     //let storage_box: Box<dyn Storage> = Box::new(GoStorage::new(unsafe { *Box::from_raw(res_store) }));
        //
        // let querier_box = Box::new(GoQuerier1::default());
        // let storage_box = Box::new(GoStorage1::default());
        // return (byte_array, querier_box, storage_box);

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
            //return (byte_array, querier_box, storage_box)
    }

    fn delegate_call<A: BackendApi + 'static, S: Storage, Q: Querier>(&self, env: &Environment<A, S, Q>,
                                                                      contract_address: String,
                                                                      info: &MessageInfo,
                                                                      call_msg: &[u8],
                                                                      block_env: &Env,
                                                                      gas_limit: u64
    ) -> (VmResult<Vec<u8>>, GasInfo) {
        env::set_var("RUST_BACKTRACE", "1");
        println!("wasmvm delegate_call contract_address: {:?}, caller address {:?}, sender address {:?}",
                 contract_address, env.delegate_contract_addr.clone(), info.sender.clone());

        // get contract code content
        let c_str = CString::new(contract_address).expect("Failed to create CString");
        let c_str_ptr = c_str.into_raw() as *mut c_char;
        let mut res_code_hash0 = UnmanagedVector::default();
        let mut res_store0: *mut Db = std::ptr::null_mut();
        let mut res_querier0: *mut GoQuerier = std::ptr::null_mut();

        // 1. 先获取处被调用合约的code
        let go_result: GoError = (self.vtable.generate_call_info)(
            self.state,
            c_str_ptr,
            &mut res_code_hash0 as *mut UnmanagedVector,
            &mut res_store0,
            &mut res_querier0,
        ).into();

        unsafe {
            // 释放c_string_ptr
            let _ = CString::from_raw(c_str_ptr);
        }
        let res_code_hash0 = res_code_hash0.consume();
        let bin_res_code_hash: Vec<u8> = res_code_hash0.unwrap_or_default();
        let mut byte_array: [u8; 32] = [0; 32];
        byte_array.copy_from_slice(&bin_res_code_hash[..32]);


        // 2. 获取caller的存储上下文
        let caller_address = env.delegate_contract_addr.clone().into_string();
        let c_string = CString::new(caller_address).expect("Failed to create CString");
        let c_string_ptr = c_string.into_raw() as *mut c_char;
        let mut res_code_hash = UnmanagedVector::default();
        let mut res_store: *mut Db = std::ptr::null_mut();
        let mut res_querier: *mut GoQuerier = std::ptr::null_mut();

        let go_result: GoError = (self.vtable.generate_call_info)(
            self.state,
            c_string_ptr,
            &mut res_code_hash as *mut UnmanagedVector,
            &mut res_store,
            &mut res_querier,
        ).into();

        unsafe {
            // 释放c_string_ptr
            let _ = CString::from_raw(c_string_ptr);
        }

        if res_store.is_null() {
            println!("wasmvm generate_call_info res_store is null");
        }

        if res_querier.is_null() {
            println!("wasmvm generate_call_info res_querier is null");
        }
        let querier = unsafe{(*res_querier).clone()};
        let storage = GoStorage::new(unsafe{(*res_store).clone()});

        let mut res_api: *mut GoApi = std::ptr::null_mut();
        let mut res_cache: *mut cache_t = std::ptr::null_mut();
        let go_result: GoError = (self.vtable.get_wasm_info)(
            &mut res_api,
            &mut res_cache,
        ).into();
        if res_api.is_null() {
            println!("wasmvm generate_call_info res_api is null");
        }
        if res_cache.is_null() {
            println!("wasmvm generate_call_info res_cache is null");
        }
        let api = unsafe{(*res_api).clone()};

        do_call(env, block_env, storage, querier, api, res_cache, info, call_msg, byte_array, gas_limit)
    }
}

pub fn do_call<A: BackendApi + 'static, S: Storage, Q: Querier>(
    env: &Environment<A, S, Q>,
    benv: &Env,
    storage: GoStorage,
    querier: GoQuerier,
    api: GoApi,
    cache: *mut cache_t,
    info: &MessageInfo,
    call_msg: &[u8],
    checksum: [u8; 32],
    gas_limit: u64,
) -> (VmResult<Vec<u8>>, GasInfo) {
    let backend = Backend {
        api: api,
        storage: storage,
        querier: querier
    };

    let ins_options = InstanceOptions{
        gas_limit: gas_limit,
        print_debug: env.print_debug,
    };

    // println!("wasmvm do_call 1");
    // let path = PathBuf::from("_cache_evm/data/wasm");
    // let options = CacheOptions {
    //     base_dir: path,
    //     supported_features: features_from_csv("iterator,staking,stargate"),
    //     memory_cache_size:  Size::mebi(100),
    //     instance_memory_limit:  Size::mebi(32),
    // };
    // println!("wasmvm do_call 2");
    // let cache = unsafe { Cache::new(options) }.unwrap();

    let cache = unsafe { &mut *(cache as *mut Cache<GoApi, GoStorage, GoQuerier>) };


    //let cache = get_global_cache();


    //println!("wasmvm do_call 3, {:?} {:?}", ins_options.gas_limit, checksum);
    let param = InternalCallParam {
        call_depth: env.call_depth + 1,
        sender_addr: env.sender_addr.clone(),
        delegate_contract_addr: env.delegate_contract_addr.clone()
    };
    let mut new_instance = cache.get_instance_ex(&Checksum::from(checksum), backend, ins_options, param);
    match new_instance {
        Ok(mut ins) => {
            let benv = to_vec(benv).unwrap();
            let info = to_vec(info).unwrap();
            println!("start the call_execute_raw");
            let result = call_execute_raw(&mut ins, &benv, &info, call_msg);
            let gas_used = gas_limit - ins.get_gas_left();
            let gas_externally_used = ins.get_externally_used_gas();
            let gas_cost       = gas_used - gas_externally_used;
                println!("the call_execute result is gas_limit {} gas_used {}, gas_externally_used {}", gas_limit, gas_used, gas_externally_used);
            (result, GasInfo::new(gas_cost, gas_externally_used))
        }
        Err(err) => {
            (Err(err), GasInfo::with_cost(0))
        }
    }
    // let result= call_execute::<_, _, _, Empty>(&mut new_instance, benv, info, call_msg).map_err(|errno|{
    //     println!("the call_execute result is {:?}", &errno);
    //     return errno;
    // })?;
    //let nn = new_instance.get_gas_left();
    //println!("the call_execute result is {} ", nn);
    // let serialized = to_vec(&result).unwrap();
    // new_instance.write_to_contract(&serialized)

    // let benv = to_vec(benv).unwrap();
    // let info = to_vec(info).unwrap();
    // println!("start the call_execute_raw");
    // let result = call_execute_raw(&mut new_instance, &benv, &info, call_msg);
    // let gas_left = new_instance.get_gas_left();
    // let gas_used = gas_limit - gas_left;
    // println!("the call_execute result is {} ", gas_used);
    // (result, gas_used)
}

// #[derive(Default)]
// pub struct GoQuerier1 {
//     is_none: bool,
// }
//
// impl Querier for GoQuerier1 {
//     fn query_raw(
//         &self,
//         request: &[u8],
//         gas_limit: u64,
//     ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
//         todo!()
//     }
//     fn generate_call_info(&self, contract_address: String) -> ([u8; 32], Box<dyn Querier>, Box<dyn Storage>) {
//         todo!()
//     }
// }
//
// #[derive(Default)]
// pub struct GoStorage1 {
//     is_none: bool,
// }
//
// impl Storage for GoStorage1 {
//     fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
//         todo!()
//     }
//     fn scan(
//         &mut self,
//         start: Option<&[u8]>,
//         end: Option<&[u8]>,
//         order: Order,
//     ) -> BackendResult<u32> {
//         todo!()
//     }
//     fn next(&mut self, iterator_id: u32) -> BackendResult<Option<Record>> {
//         todo!()
//     }
//     fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
//         todo!()
//     }
//     fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
//         todo!()
//     }
// }
