use drink::MinimalSandbox;
use simulation::{call_contract, deploy_contract, report_gas, NEW_SELECTOR};

const FLIP_SELECTOR: [u8; 4] = [99, 58, 165, 81];

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

    let (riscv_gas, _) = call_contract(&mut sandbox, riscv_instance, FLIP_SELECTOR.to_vec(), None);
    let (wasm_gas, _) = call_contract(&mut sandbox, wasm_instance, FLIP_SELECTOR.to_vec(), None);

    report_gas(riscv_gas, wasm_gas, "flipper", "flip");
}
