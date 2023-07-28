use cosmwasm_vm::{BackendApi, BackendError, BackendResult, GasInfo, Querier, Storage, VmError};

use crate::error::GoError;
use crate::memory::{U8SliceView, UnmanagedVector};

use crate::db::Db;
use crate::{cache_t, GoQuerier, GoStorage};
use cosmwasm_std::{to_vec, Addr, Env, MessageInfo};
use cosmwasm_vm::{call_execute_raw, Backend, Cache, Checksum, Environment};
use cosmwasm_vm::{InstanceOptions, InternalCallParam, VmResult};

// this represents something passed in from the caller side of FFI
// in this case a struct with go function pointers
#[repr(C)]
pub struct api_t {
    _private: [u8; 0],
}

// These functions should return GoError but because we don't trust them here, we treat the return value as i32
// and then check it when converting to GoError manually
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GoApi_vtable {
    pub humanize_address: extern "C" fn(
        *const api_t,
        U8SliceView,
        *mut UnmanagedVector, // human output
        *mut UnmanagedVector, // error message output
        *mut u64,
    ) -> i32,
    pub canonicalize_address: extern "C" fn(
        *const api_t,
        U8SliceView,
        *mut UnmanagedVector, // canonical output
        *mut UnmanagedVector, // error message output
        *mut u64,
    ) -> i32,
    pub get_call_info: extern "C" fn(
        *const api_t,
        *mut u64,    // used gas
        U8SliceView, // contract address
        U8SliceView, // store address
        *mut UnmanagedVector,
        *mut *mut Db,
        *mut *mut GoQuerier,
        *mut u64,             // the callID
        *mut UnmanagedVector, // error message output
    ) -> i32,
    pub get_wasm_info: extern "C" fn(
        *mut *mut cache_t,
        *mut UnmanagedVector, // error message output
    ) -> i32,
    pub release: extern "C" fn(
        u64, // callID
    ) -> i32,
    pub transfer_coins: extern "C" fn(
        *const api_t,
        *mut u64,             // used gas
        U8SliceView,          // contract address
        U8SliceView,          // caller
        U8SliceView,          // coins
        *mut UnmanagedVector, // error message output
    ) -> i32,
    pub contract_external: extern "C" fn(
        *const api_t,
        u64,
        *mut u64,
        U8SliceView,
        *mut UnmanagedVector, // result output
        *mut UnmanagedVector, // error message output
    ) -> i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GoApi {
    pub state: *const api_t,
    pub vtable: GoApi_vtable,
}

// We must declare that these are safe to Send, to use in wasm.
// The known go caller passes in immutable function pointers, but this is indeed
// unsafe for possible other callers.
//
// see: https://stackoverflow.com/questions/50258359/can-a-struct-containing-a-raw-pointer-implement-send-and-be-ffi-safe
unsafe impl Send for GoApi {}

impl BackendApi for GoApi {
    fn canonical_address(&self, human: &str) -> BackendResult<Vec<u8>> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_error: GoError = (self.vtable.canonicalize_address)(
            self.state,
            U8SliceView::new(Some(human.as_bytes())),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
            &mut used_gas as *mut u64,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

        let gas_info = GasInfo::with_cost(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || format!("Failed to canonicalize the address: {}", human);
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        let result = output.ok_or_else(|| BackendError::unknown("Unset output"));
        (result, gas_info)
    }

    fn human_address(&self, canonical: &[u8]) -> BackendResult<String> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_error: GoError = (self.vtable.humanize_address)(
            self.state,
            U8SliceView::new(Some(canonical)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
            &mut used_gas as *mut u64,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

        let gas_info = GasInfo::with_cost(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || {
            format!(
                "Failed to humanize the address: {}",
                hex::encode_upper(canonical)
            )
        };
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        let result = output
            .ok_or_else(|| BackendError::unknown("Unset output"))
            .and_then(|human_data| String::from_utf8(human_data).map_err(BackendError::from));
        (result, gas_info)
    }

    fn call<A: BackendApi, S: Storage, Q: Querier>(
        &self,
        env1: &Environment<A, S, Q>,
        contract_address: String,
        info: &MessageInfo,
        call_msg: &[u8],
        block_env: &Env,
        gas_limit: u64,
    ) -> (VmResult<Vec<u8>>, GasInfo) {
        let mut tc_used_gas = 0_u64;
        // need check transfer
        if info.funds.len() != 0 {
            let mut error_msg = UnmanagedVector::default();
            let coins = to_vec(&info.funds).unwrap();
            let go_result: GoError = (self.vtable.transfer_coins)(
                self.state,
                &mut tc_used_gas as *mut u64,
                U8SliceView::new(Some(contract_address.as_bytes())),
                U8SliceView::new(Some(info.sender.clone().as_bytes())),
                U8SliceView::new(Some(coins.as_slice())),
                &mut error_msg as *mut UnmanagedVector,
            )
            .into();
            let default = || {
                format!(
                    "Call failed to transferCoins contract address: {}",
                    contract_address
                )
            };
            unsafe {
                if let Err(err) = go_result.into_result(error_msg, default) {
                    return (
                        Err(VmError::BackendErr { source: err }),
                        GasInfo::with_externally_used(tc_used_gas),
                    );
                }
            }
        }

        let mut res_code_hash = UnmanagedVector::default();
        let mut res_store: *mut Db = std::ptr::null_mut();
        let mut res_querier: *mut GoQuerier = std::ptr::null_mut();
        let mut error_msg = UnmanagedVector::default();
        let mut gci_used_gas = 0_u64;
        let mut call_id = 0_u64;

        let go_result: GoError = (self.vtable.get_call_info)(
            self.state,
            &mut gci_used_gas as *mut u64,
            U8SliceView::new(Some(contract_address.as_bytes())),
            U8SliceView::new(Some(contract_address.as_bytes())),
            &mut res_code_hash as *mut UnmanagedVector,
            &mut res_store,
            &mut res_querier,
            &mut call_id as *mut u64,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();

        let mut ret_gas_uesd = GasInfo::with_externally_used(gci_used_gas + tc_used_gas);

        let default = || {
            format!(
                "Call failed to get_call_info contract address: {}",
                contract_address
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(VmError::BackendErr { source: err }), ret_gas_uesd);
            }
        }

        if res_store.is_null() || res_querier.is_null() || res_code_hash.is_none() {
            return (
                Err(VmError::BackendErr {
                    source: BackendError::unknown(
                        "res_store, res_querier or res_code_hash is null",
                    ),
                }),
                ret_gas_uesd,
            );
        }

        // We destruct the UnmanagedVector here, no matter if we need the data.
        let res_code_hash = res_code_hash.consume();
        let bin_res_code_hash: Vec<u8> = res_code_hash.unwrap_or_default();
        let mut byte_array: [u8; 32] = [0; 32];
        byte_array.copy_from_slice(&bin_res_code_hash[..32]);

        let querier = unsafe { (*res_querier).clone() };
        let storage = GoStorage::new(unsafe { (*res_store).clone() });

        let mut res_cache: *mut cache_t = std::ptr::null_mut();
        let mut error_msg = UnmanagedVector::default();

        let go_result: GoError =
            (self.vtable.get_wasm_info)(&mut res_cache, &mut error_msg as *mut UnmanagedVector)
                .into();

        let default = || {
            format!(
                "Call failed to get_wasm_info contract address: {}",
                contract_address
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(VmError::BackendErr { source: err }), ret_gas_uesd);
            }
        }
        if res_cache.is_null() {
            return (
                Err(VmError::BackendErr {
                    source: BackendError::unknown("res_api or res_cache is null"),
                }),
                ret_gas_uesd,
            );
        }

        let api = self.clone();
        let (result, gas_info) = do_call(
            env1,
            block_env,
            storage,
            querier,
            api,
            res_cache,
            info,
            call_msg,
            byte_array,
            gas_limit,
            Addr::unchecked(contract_address.clone()),
        );

        ret_gas_uesd += gas_info;
        (self.vtable.release)(call_id);

        (result, ret_gas_uesd)
    }

    fn delegate_call<A: BackendApi, S: Storage, Q: Querier>(
        &self,
        env: &Environment<A, S, Q>,
        contract_address: String,
        info: &MessageInfo,
        call_msg: &[u8],
        block_env: &Env,
        gas_limit: u64,
    ) -> (VmResult<Vec<u8>>, GasInfo) {
        let mut res_code_hash = UnmanagedVector::default();
        let mut res_store: *mut Db = std::ptr::null_mut();
        let mut res_querier: *mut GoQuerier = std::ptr::null_mut();
        let mut error_msg = UnmanagedVector::default();
        let mut gci_used_gas = 0_u64;
        let mut call_id = 0_u64;

        let go_result: GoError = (self.vtable.get_call_info)(
            self.state,
            &mut gci_used_gas as *mut u64,
            U8SliceView::new(Some(contract_address.as_bytes())),
            U8SliceView::new(Some(env.delegate_contract_addr.clone().as_bytes())),
            &mut res_code_hash as *mut UnmanagedVector,
            &mut res_store,
            &mut res_querier,
            &mut call_id as *mut u64,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();

        let mut ret_gas_uesd = GasInfo::with_externally_used(gci_used_gas);

        let default = || {
            format!(
                "Delegate call failed to get_call_info contract address: {} store address: {}",
                contract_address,
                env.delegate_contract_addr.clone().into_string()
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(VmError::BackendErr { source: err }), ret_gas_uesd);
            }
        }

        if res_store.is_null() || res_querier.is_null() || res_code_hash.is_none() {
            return (
                Err(VmError::BackendErr {
                    source: BackendError::unknown(
                        "res_store, res_querier or res_code_hash is null",
                    ),
                }),
                ret_gas_uesd,
            );
        }

        let res_code_hash = res_code_hash.consume();
        let bin_res_code_hash: Vec<u8> = res_code_hash.unwrap_or_default();
        let mut byte_array: [u8; 32] = [0; 32];
        byte_array.copy_from_slice(&bin_res_code_hash[..32]);

        let querier = unsafe { (*res_querier).clone() };
        let storage = GoStorage::new(unsafe { (*res_store).clone() });

        let mut res_cache: *mut cache_t = std::ptr::null_mut();
        let mut error_msg = UnmanagedVector::default();
        let go_result: GoError =
            (self.vtable.get_wasm_info)(&mut res_cache, &mut error_msg as *mut UnmanagedVector)
                .into();

        let default = || {
            format!(
                "Delegate call failed to get_wasm_info contract address: {} store address: {}",
                contract_address,
                env.delegate_contract_addr.clone().into_string()
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(VmError::BackendErr { source: err }), ret_gas_uesd);
            }
        }
        if res_cache.is_null() {
            return (
                Err(VmError::BackendErr {
                    source: BackendError::unknown("res_api or res_cache is null"),
                }),
                ret_gas_uesd,
            );
        }

        let api = self.clone();

        let (result, gas_info) = do_call(
            env,
            block_env,
            storage,
            querier,
            api,
            res_cache,
            info,
            call_msg,
            byte_array,
            gas_limit,
            env.delegate_contract_addr.clone(),
        );

        ret_gas_uesd += gas_info;
        (self.vtable.release)(call_id);

        (result, ret_gas_uesd)
    }

    fn new_contract(&self, request: &[u8], gas_limit: u64) -> BackendResult<String> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_error: GoError = (self.vtable.contract_external)(
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
                "Failed to new contract: {}",
                String::from_utf8_lossy(request)
            )
        };
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        let result = output
            .ok_or_else(|| BackendError::unknown("Unset output"))
            .and_then(|addr_data| String::from_utf8(addr_data).map_err(BackendError::from));
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
        querier: querier,
    };

    let ins_options = InstanceOptions {
        gas_limit: gas_limit,
        print_debug: env.print_debug,
        write_cost_flat: env.gas_config_info.write_cost_flat,
        write_cost_per_byte: env.gas_config_info.write_cost_per_byte,
        delete_cost: env.gas_config_info.delete_cost,
        gas_mul: env.gas_config_info.gas_mul,
    };

    let cache = unsafe { &mut *(cache as *mut Cache<GoApi, GoStorage, GoQuerier>) };
    let param = InternalCallParam {
        call_depth: env.call_depth + 1,
        sender_addr: info.sender.clone(),
        delegate_contract_addr: delegate_contract_addr,
    };
    let new_instance =
        cache.get_instance_ex(&Checksum::from(checksum), backend, ins_options, param);
    match new_instance {
        Ok(mut ins) => {
            let benv = to_vec(benv).unwrap();
            let info = to_vec(info).unwrap();
            let result = call_execute_raw(&mut ins, &benv, &info, call_msg);
            let gas_rep = ins.create_gas_report();
            ins.recycle();
            (
                result,
                GasInfo::new(gas_rep.used_internally, gas_rep.used_externally),
            )
        }
        Err(err) => (Err(err), GasInfo::with_cost(0)),
    }
}
