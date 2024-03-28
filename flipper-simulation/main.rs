use drink::{
    AccountId32, contracts_api::ContractAPI,
    MinimalSandbox, pallet_contracts::Determinism, Sandbox, Weight,
};
use num_format::*;

const NEW_SELECTOR: [u8; 4] = [155u8, 174u8, 157u8, 94u8];
const FLIP_SELECTOR: [u8; 4] = [99, 58, 165, 81];
const GAS_LIMIT: Weight = Weight::from_parts(500_000_000_000, 500_000_000_000);

fn deploy_contract(
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

fn call_contract(
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

fn report_gas(riscv_gas: u64, wasm_gas: u64, contract: &'static str, action: &'static str) {
    println!("----------------------------------------");
    println!("[{contract}][riscv][{action}]: {}", format_gas(&riscv_gas));
    println!("[{contract}][wasm] [{action}]: {}", format_gas(&wasm_gas));
    println!("Speedup: {:.0}%", (riscv_gas as f64 / wasm_gas as f64) * 100f64);
}

fn main() {
    let mut sandbox = MinimalSandbox::default();

    let riscv_bytes = include_bytes!("../flipper/artifacts/flipper.riscv");
    let wasm_bytes = include_bytes!("../flipper/artifacts/flipper.wasm");

    let instantiation_data = [NEW_SELECTOR.to_vec(), vec![1]].concat();

    let (riscv_gas, riscv_instance) = deploy_contract(
        &mut sandbox,
        instantiation_data.clone(),
        riscv_bytes.to_vec(),
    );
    let (wasm_gas, wasm_instance) = deploy_contract(
        &mut sandbox,
        instantiation_data.clone(),
        wasm_bytes.to_vec(),
    );

    report_gas(riscv_gas, wasm_gas, "flipper", "deploy");

    let (riscv_gas, _) = call_contract(&mut sandbox, riscv_instance, FLIP_SELECTOR.to_vec());
    let (wasm_gas, _) = call_contract(&mut sandbox, wasm_instance, FLIP_SELECTOR.to_vec());

    report_gas(riscv_gas, wasm_gas, "flipper", "flip");
}
