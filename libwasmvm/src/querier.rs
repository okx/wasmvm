use cosmwasm_std::{Addr, Binary, ContractResult, SystemError, SystemResult};
use cosmwasm_vm::{BackendError, BackendResult, GasInfo, Querier, Storage, VmError};

use crate::error::GoError;
use crate::memory::{U8SliceView, UnmanagedVector};
use crate::db::{Db, db_t};
use crate::storage::GoStorage;
use crate::api::GoApi;
use crate::cache::{cache_t};

use cosmwasm_vm::{call_execute_raw, Backend, Cache, Checksum, Environment, BackendApi};
use cosmwasm_vm::{VmResult, InstanceOptions, InternalCallParam};
use cosmwasm_std::{MessageInfo, to_vec, Env};

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
    pub get_call_info: extern "C" fn (
        *const querier_t,
        U8SliceView,    // contract address
        U8SliceView,    // store address
        *mut UnmanagedVector,
        *mut *mut Db,
        *mut *mut GoQuerier,
        *mut UnmanagedVector, // error message output
    ) -> i32,
    pub get_wasm_info: extern "C" fn (
        *mut *mut GoApi,
        *mut *mut cache_t,
        *mut UnmanagedVector, // error message output
    ) -> i32,
    pub release: extern "C" fn(
        *mut db_t,
    )-> i32,
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

    fn call<A: BackendApi, S: Storage, Q: Querier>(&self, env1: &Environment<A, S, Q>,
                          contract_address: String,
                          info: &MessageInfo,
                          call_msg: &[u8],
                          block_env: &Env,
                          gas_limit: u64
    ) -> (VmResult<Vec<u8>>, GasInfo) {
        let mut res_code_hash = UnmanagedVector::default();
        let mut res_store: *mut Db = std::ptr::null_mut();
        let mut res_querier: *mut GoQuerier = std::ptr::null_mut();
        let mut error_msg = UnmanagedVector::default();

        let go_result: GoError = (self.vtable.get_call_info)(
            self.state,
            U8SliceView::new(Some(contract_address.as_bytes())),
            U8SliceView::new(Some(contract_address.as_bytes())),
            &mut res_code_hash as *mut UnmanagedVector,
            &mut res_store,
            &mut res_querier,
            &mut error_msg as *mut UnmanagedVector,
        ).into();

        let default = || {
            format!(
                "Call failed to get_call_info contract address: {}",
                contract_address
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(VmError::BackendErr { source: err }), GasInfo::with_cost(0));
            }
        }

        if res_store.is_null() || res_querier.is_null() || res_code_hash.is_none(){
            return (Err(VmError::BackendErr {source: BackendError::unknown("res_store, res_querier or res_code_hash is null")}), GasInfo::with_cost(0));
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
        let mut error_msg = UnmanagedVector::default();

        let go_result: GoError = (self.vtable.get_wasm_info)(
            &mut res_api,
            &mut res_cache,
            &mut error_msg as *mut UnmanagedVector,
        ).into();

        let default = || {
            format!(
                "Call failed to get_wasm_info contract address: {}",
                contract_address
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(VmError::BackendErr { source: err }), GasInfo::with_cost(0));
            }
        }
        if res_api.is_null() || res_cache.is_null() {
            return (Err(VmError::BackendErr { source: BackendError::unknown("res_api or res_cache is null") }), GasInfo::with_cost(0));
        }

        let api = unsafe{(*res_api).clone()};
        let (result, gas_info) = do_call(env1, block_env, storage, querier, api, res_cache, info, call_msg, byte_array, gas_limit, Addr::unchecked(contract_address.clone()));

        (self.vtable.release)(unsafe{(*res_store).state});

        (result, gas_info)
    }

    fn delegate_call<A: BackendApi, S: Storage, Q: Querier>(&self, env: &Environment<A, S, Q>,
                                                                      contract_address: String,
                                                                      info: &MessageInfo,
                                                                      call_msg: &[u8],
                                                                      block_env: &Env,
                                                                      gas_limit: u64
    ) -> (VmResult<Vec<u8>>, GasInfo) {
        let mut res_code_hash = UnmanagedVector::default();
        let mut res_store: *mut Db = std::ptr::null_mut();
        let mut res_querier: *mut GoQuerier = std::ptr::null_mut();
        let mut error_msg = UnmanagedVector::default();

        let go_result: GoError = (self.vtable.get_call_info)(
            self.state,
            U8SliceView::new(Some(contract_address.as_bytes())),
            U8SliceView::new(Some(env.delegate_contract_addr.clone().as_bytes())),
            &mut res_code_hash as *mut UnmanagedVector,
            &mut res_store,
            &mut res_querier,
            &mut error_msg as *mut UnmanagedVector,
        ).into();

        let default = || {
            format!(
                "Delegate call failed to get_call_info contract address: {} store address: {}",
                contract_address,
                env.delegate_contract_addr.clone().into_string()
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(VmError::BackendErr { source: err }), GasInfo::with_cost(0));
            }
        }

        if res_store.is_null() || res_querier.is_null() || res_code_hash.is_none(){
            return (Err(VmError::BackendErr {source: BackendError::unknown("res_store, res_querier or res_code_hash is null")}), GasInfo::with_cost(0));
        }

        let res_code_hash = res_code_hash.consume();
        let bin_res_code_hash: Vec<u8> = res_code_hash.unwrap_or_default();
        let mut byte_array: [u8; 32] = [0; 32];
        byte_array.copy_from_slice(&bin_res_code_hash[..32]);

        let querier = unsafe{(*res_querier).clone()};
        let storage = GoStorage::new(unsafe{(*res_store).clone()});

        let mut res_api: *mut GoApi = std::ptr::null_mut();
        let mut res_cache: *mut cache_t = std::ptr::null_mut();
        let mut error_msg = UnmanagedVector::default();
        let go_result: GoError = (self.vtable.get_wasm_info)(
            &mut res_api,
            &mut res_cache,
            &mut error_msg as *mut UnmanagedVector,
        ).into();

        let default = || {
            format!(
                "Delegate call failed to get_wasm_info contract address: {} store address: {}",
                contract_address,
                env.delegate_contract_addr.clone().into_string()
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(VmError::BackendErr { source: err }), GasInfo::with_cost(0));
            }
        }
        if res_api.is_null() || res_cache.is_null() {
            return (Err(VmError::BackendErr { source: BackendError::unknown("res_api or res_cache is null") }), GasInfo::with_cost(0));
        }

        let api = unsafe{(*res_api).clone()};
        let (result, gas_info) = do_call(env, block_env, storage, querier, api, res_cache, info, call_msg, byte_array, gas_limit, env.delegate_contract_addr.clone());

        (self.vtable.release)(unsafe{(*res_store).state});

        (result, gas_info)
    }
}

pub fn do_call<A: BackendApi, S: Storage, Q: Querier>(
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
    delegate_contract_addr: Addr,
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

    let cache = unsafe { &mut *(cache as *mut Cache<GoApi, GoStorage, GoQuerier>) };
    let param = InternalCallParam {
        call_depth: env.call_depth + 1,
        sender_addr: env.sender_addr.clone(),
        delegate_contract_addr: delegate_contract_addr
    };
    let new_instance = cache.get_instance_ex(&Checksum::from(checksum), backend, ins_options, param);
    match new_instance {
        Ok(mut ins) => {
            let benv = to_vec(benv).unwrap();
            let info = to_vec(info).unwrap();
            let result = call_execute_raw(&mut ins, &benv, &info, call_msg);
            let gas_used = gas_limit - ins.get_gas_left();
            let gas_externally_used = ins.get_externally_used_gas();
            let gas_cost       = gas_used - gas_externally_used;
            (result, GasInfo::new(gas_cost, gas_externally_used))
        }
        Err(err) => {
            (Err(err), GasInfo::with_cost(0))
        }
    }
}
