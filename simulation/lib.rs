use drink::{AccountId32, MinimalSandbox, Sandbox, Weight};
use drink::contracts_api::ContractAPI;
use drink::pallet_contracts::Determinism;
use num_format::{Buffer, Locale, ToFormattedStr};

pub const NEW_SELECTOR: [u8; 4] = [155u8, 174u8, 157u8, 94u8];
pub const GAS_LIMIT: Weight = Weight::from_parts(500_000_000_000, 500_000_000_000);

pub fn deploy_contract(
    sandbox: &mut MinimalSandbox,
    data: Vec<u8>,
    code: Vec<u8>,
) -> (u64, AccountId32) {
    let main_result = sandbox.deploy_contract(
        code,
        0,
        data,
        vec![],
        MinimalSandbox::default_actor(),
        GAS_LIMIT,
        None,
    );
    let instantiation_result = main_result.result.unwrap();
    assert!(!instantiation_result.result.did_revert());

    (
        main_result.gas_consumed.ref_time(),
        instantiation_result.account_id,
    )
}

pub fn call_contract(
    sandbox: &mut MinimalSandbox,
    contract: AccountId32,
    data: Vec<u8>,
) -> (u64, Vec<u8>) {
    let main_result = sandbox.call_contract(
        contract,
        0,
        data,
        MinimalSandbox::default_actor(),
        GAS_LIMIT,
        None,
        Determinism::Enforced,
    );
    let call_result = main_result.result.unwrap();
    assert!(!call_result.did_revert());

    (main_result.gas_consumed.ref_time(), call_result.data)
}

fn format_gas<G: ToFormattedStr>(gas: &G) -> String {
    let mut buf = Buffer::default();
    buf.write_formatted(gas, &Locale::en);
    format!("{}", buf.as_str())
}

pub fn report_gas(riscv_gas: u64, wasm_gas: u64, contract: &'static str, action: &'static str) {
    println!("----------------------------------------");
    println!("[{contract}][riscv][{action}]: {}", format_gas(&riscv_gas));
    println!("[{contract}][wasm] [{action}]: {}", format_gas(&wasm_gas));
    println!("Speedup: {:.0}%", (riscv_gas as f64 / wasm_gas as f64) * 100f64);
}
