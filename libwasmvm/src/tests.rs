#![cfg(test)]

use std::env;
use cosmwasm_std::{Addr, Env, MessageInfo};
use tempfile::TempDir;

use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info, mock_instance_with_gas_limit};
use cosmwasm_vm::{call_execute_raw, call_instantiate_raw, capabilities_from_csv, to_vec, Cache, CacheOptions, InstanceOptions, Size,
                  Checksum, Storage, BackendApi, Querier, GasInfo, VmResult, InternalCallParam, Backend};

static CYBERPUNK: &[u8] = include_bytes!("../../testdata/cyberpunk.wasm");
const PRINT_DEBUG: bool = false;
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);
const MEMORY_LIMIT: Size = Size::mebi(32);
const GAS_LIMIT: u64 = 200_000_000_000; // ~0.2ms

#[test]
fn handle_cpu_loop_with_cache() {
    let backend = mock_backend(&[]);
    let options = CacheOptions {
        base_dir: TempDir::new().unwrap().path().to_path_buf(),
        available_capabilities: capabilities_from_csv("staking"),
        memory_cache_size: MEMORY_CACHE_SIZE,
        instance_memory_limit: MEMORY_LIMIT,
    };
    let cache = unsafe { Cache::new(options) }.unwrap();

    let options = InstanceOptions {
        gas_limit: GAS_LIMIT,
        print_debug: PRINT_DEBUG,
    };

    // store code
    let checksum = cache.save_wasm(CYBERPUNK).unwrap();

    // instantiate
    let env = mock_env();
    let info = mock_info("creator", &[]);
    let mut instance = cache.get_instance(&checksum, backend, options).unwrap();
    let raw_env = to_vec(&env).unwrap();
    let raw_info = to_vec(&info).unwrap();
    let res = call_instantiate_raw(&mut instance, &raw_env, &raw_info, b"{}");
    let gas_left = instance.get_gas_left();
    let gas_used = options.gas_limit - gas_left;
    println!("Init gas left: {}, used: {}", gas_left, gas_used);
    assert!(res.is_ok());
    let backend = instance.recycle().unwrap();

    // execute
    let mut instance = cache.get_instance(&checksum, backend, options).unwrap();
    let raw_msg = br#"{"cpu_loop":{}}"#;
    let res = call_execute_raw(&mut instance, &raw_env, &raw_info, raw_msg);
    let gas_left = instance.get_gas_left();
    let gas_used = options.gas_limit - gas_left;
    println!("Handle gas left: {}, used: {}", gas_left, gas_used);
    assert!(res.is_err());
    assert_eq!(gas_left, 0);
    let _ = instance.recycle();
}

#[test]
fn handle_cpu_loop_no_cache() {
    let gas_limit = GAS_LIMIT;
    let mut instance = mock_instance_with_gas_limit(CYBERPUNK, gas_limit);

    // instantiate
    let env = mock_env();
    let info = mock_info("creator", &[]);
    let raw_env = to_vec(&env).unwrap();
    let raw_info = to_vec(&info).unwrap();
    let res = call_instantiate_raw(&mut instance, &raw_env, &raw_info, b"{}");
    let gas_left = instance.get_gas_left();
    let gas_used = gas_limit - gas_left;
    println!("Init gas left: {}, used: {}", gas_left, gas_used);
    assert!(res.is_ok());

    // execute
    let raw_msg = br#"{"cpu_loop":{}}"#;
    let res = call_execute_raw(&mut instance, &raw_env, &raw_info, raw_msg);
    let gas_left = instance.get_gas_left();
    let gas_used = gas_limit - gas_left;
    println!("Handle gas left: {}, used: {}", gas_left, gas_used);
    assert!(res.is_err());
    assert_eq!(gas_left, 0);
}

#[test]
fn handle_do_call() {
    env::set_var("RUST_BACKTRACE", "full");
    let backend = mock_backend(&[]);
    let options = CacheOptions {
        base_dir: TempDir::new().unwrap().path().to_path_buf(),
        available_capabilities: capabilities_from_csv("staking"),
        memory_cache_size: MEMORY_CACHE_SIZE,
        instance_memory_limit: MEMORY_LIMIT,
    };
    let cache = unsafe { Cache::new(options) }.unwrap();

    let options = InstanceOptions {
        gas_limit: GAS_LIMIT,
        print_debug: PRINT_DEBUG,
    };

    // store code
    let checksum = cache.save_wasm(CYBERPUNK).unwrap();

    // instantiate
    let env = mock_env();
    let info = mock_info("creator", &[]);
    let mut instance = cache.get_instance(&checksum, backend, options).unwrap();
    let raw_env = to_vec(&env).unwrap();
    let raw_info = to_vec(&info).unwrap();
    let res = call_instantiate_raw(&mut instance, &raw_env, &raw_info, b"{}");
    let gas_left = instance.get_gas_left();
    let gas_used = options.gas_limit - gas_left;
    println!("Init gas left: {}, used: {}", gas_left, gas_used);
    assert!(res.is_ok());
    let backend = instance.recycle().unwrap();

    // execute
    let raw_msg = br#"{"cpu_loop":{}}"#;
    let data  = Vec::from(checksum);
    let mut byte_array: [u8; 32] = [0; 32];
    byte_array.copy_from_slice(&data[..32]);


    let (res, gas_info) = mock_do_call(&env, backend, &cache , &info, raw_msg, byte_array,
             options.gas_limit, options.print_debug, 1, cosmwasm_std::Addr::unchecked(""), cosmwasm_std::Addr::unchecked(""));
    assert!(res.is_err());
    println!("the gas_info is {:?}", gas_info);
}

pub fn mock_do_call<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    benv: &Env,
    backend: Backend<A, S, Q>,
    cache: &Cache<A, S, Q>,
    info: &MessageInfo,
    call_msg: &[u8],
    checksum: [u8; 32],
    gas_limit: u64,
    print_debug: bool,
    call_depth: u32,
    sender_addr: Addr,
    delegate_contract_addr: Addr,
) -> (VmResult<Vec<u8>>, GasInfo) {
    let ins_options = InstanceOptions{
        gas_limit: gas_limit,
        print_debug: print_debug,
    };

    let param = InternalCallParam {
        call_depth: call_depth + 1,
        sender_addr: sender_addr,
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
