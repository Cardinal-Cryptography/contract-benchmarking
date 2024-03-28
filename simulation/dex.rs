use drink::{
    balance_api::BalanceAPI,
    contracts_api::ContractAPI,
    frame_support::{
        pallet_prelude::{Decode, Encode},
        sp_runtime::{app_crypto::sp_core::Hasher, traits::BlakeTwo256},
    },
    pallet_contracts::Determinism,
    timestamp_api::TimestampAPI,
    AccountId32, MinimalSandbox, Sandbox,
};

use simulation::{call_contract, instantiate_contract, report_gas, NEW_SELECTOR};

fn psp22_riscv_bytes() -> Vec<u8> {
    include_bytes!("../dex/artifacts/psp22.riscv").to_vec()
}
fn psp22_wasm_bytes() -> Vec<u8> {
    include_bytes!("../dex/artifacts/psp22.wasm").to_vec()
}
fn wrapped_riscv_bytes() -> Vec<u8> {
    include_bytes!("../dex/artifacts/wrapped_azero.riscv").to_vec()
}
fn wrapped_wasm_bytes() -> Vec<u8> {
    include_bytes!("../dex/artifacts/wrapped_azero.wasm").to_vec()
}
fn factory_riscv_bytes() -> Vec<u8> {
    include_bytes!("../dex/artifacts/factory_contract.riscv").to_vec()
}
fn factory_wasm_bytes() -> Vec<u8> {
    include_bytes!("../dex/artifacts/factory_contract.wasm").to_vec()
}
fn pair_riscv_bytes() -> Vec<u8> {
    include_bytes!("../dex/artifacts/pair_contract.riscv").to_vec()
}
fn pair_wasm_bytes() -> Vec<u8> {
    include_bytes!("../dex/artifacts/pair_contract.wasm").to_vec()
}
fn router_riscv_bytes() -> Vec<u8> {
    include_bytes!("../dex/artifacts/router_contract.riscv").to_vec()
}
fn router_wasm_bytes() -> Vec<u8> {
    include_bytes!("../dex/artifacts/router_contract.wasm").to_vec()
}

const BOB: AccountId32 = AccountId32::new([1u8; 32]);
const CHARLIE: AccountId32 = AccountId32::new([3u8; 32]);
const ICE: &str = "ICE";
const WOOD: &str = "WOOD";
const PSP22_SUPPLY: u128 = 1_000_000_000u128 * 10u128.pow(18);
const PSP22_DECIMALS: u8 = 18;

fn main() {
    let mut sandbox = MinimalSandbox::default();
    let now = sandbox.get_timestamp();
    sandbox.mint_into(&BOB, 10u128.pow(12)).unwrap();

    // ==== Upload contracts. ======================================================================
    upload_contracts(&mut sandbox);

    // ==== Instantiate contracts. =================================================================
    let (factory_riscv, factory_wasm) = factory_setup(&mut sandbox, BOB);
    let (ice_riscv, ice_wasm) = psp22_setup(&mut sandbox, ICE.to_string(), BOB);
    let (wood_riscv, wood_wasm) = psp22_setup(&mut sandbox, WOOD.to_string(), BOB);
    let (wazero_riscv, wazero_wasm) = wazero_setup(&mut sandbox);
    let (router_riscv, router_wasm) = router_setup(
        &mut sandbox,
        (factory_riscv.clone(), factory_wasm.clone()),
        (wazero_riscv.clone(), wazero_wasm.clone()),
    );

    // ==== Set fee collector to CHARLIE [3u8;32] ==================================================
    let call_data = {
        let mut data = vec![62u8, 242u8, 5u8, 167u8];
        CHARLIE.encode_to(&mut data);
        data
    };
    let (riscv_gas, _) =
        call_contract(&mut sandbox, factory_riscv.clone(), call_data.clone(), None);
    let (wasm_gas, _) = call_contract(&mut sandbox, factory_wasm.clone(), call_data, None);
    report_gas(riscv_gas, wasm_gas, "factory", "set_fee_collector");

    // ==== Increase allowance =====================================================================
    let exp = 10u128.pow(18);
    let token_amount = 1_000_000 * exp;
    let riscv_gas =
        psp22_increase_allowance(&mut sandbox, &ice_riscv, &router_riscv, u128::MAX, BOB);
    let wasm_gas = psp22_increase_allowance(&mut sandbox, &ice_wasm, &router_wasm, u128::MAX, BOB);
    report_gas(riscv_gas, wasm_gas, "psp22", "increase_allowance");

    psp22_increase_allowance(&mut sandbox, &wood_riscv, &router_riscv, u128::MAX, BOB);
    psp22_increase_allowance(&mut sandbox, &wood_wasm, &router_wasm, u128::MAX, BOB);

    // ===== Add liquidity =========================================================================
    let deadline = now + 10;
    let liquidity_minted = router_add_liquidity(
        &mut sandbox,
        (&router_riscv, &router_wasm),
        (&ice_riscv, &ice_wasm),
        (&wood_riscv, &wood_wasm),
        token_amount,
        token_amount,
        BOB,
        deadline,
    );

    // ==== Check liquidity balance ================================================================
    let data_riscv = {
        let mut data = vec![69u8, 163u8, 192u8, 246u8];
        ice_riscv.encode_to(&mut data);
        wood_riscv.encode_to(&mut data);
        data
    };
    let (riscv_gas, ice_wood_pair_riscv) =
        call_contract(&mut sandbox, factory_riscv.clone(), data_riscv, None);
    let data_wasm = {
        let mut data = vec![69u8, 163u8, 192u8, 246u8];
        ice_wasm.encode_to(&mut data);
        wood_wasm.encode_to(&mut data);
        data
    };
    let (wasm_gas, ice_wood_pair_wasm) =
        call_contract(&mut sandbox, factory_wasm.clone(), data_wasm, None);
    report_gas(riscv_gas, wasm_gas, "factory", "get_pair");

    let ice_wood_pair_riscv =
        <Result<Option<AccountId32>, ()>>::decode(&mut ice_wood_pair_riscv.as_slice())
            .unwrap()
            .unwrap()
            .unwrap();
    let ice_wood_pair_wasm =
        <Result<Option<AccountId32>, ()>>::decode(&mut ice_wood_pair_wasm.as_slice())
            .unwrap()
            .unwrap()
            .unwrap();

    let (gas_riscv, bob_lp_balance_riscv) =
        psp22_balance_of(&mut sandbox, ice_wood_pair_riscv, BOB);
    let (gas_wasm, bob_lp_balance_wasm) = psp22_balance_of(&mut sandbox, ice_wood_pair_wasm, BOB);
    assert_eq!(liquidity_minted, bob_lp_balance_riscv);
    assert_eq!(liquidity_minted, bob_lp_balance_wasm);

    report_gas(gas_riscv, gas_wasm, "psp22", "balance_of");

    // ==== Swap exact tokens for tokens ===========================================================
    let swap_amount = 10_000 * exp;

    let data_riscv = {
        let mut data = vec![175u8, 10u8, 136u8, 54u8];
        swap_amount.encode_to(&mut data);
        0u128.encode_to(&mut data);
        vec![ice_riscv, wood_riscv].encode_to(&mut data);
        BOB.encode_to(&mut data);
        deadline.encode_to(&mut data);
        data
    };
    let data_wasm = {
        let mut data = vec![175u8, 10u8, 136u8, 54u8];
        swap_amount.encode_to(&mut data);
        0u128.encode_to(&mut data);
        vec![ice_wasm, wood_wasm].encode_to(&mut data);
        BOB.encode_to(&mut data);
        deadline.encode_to(&mut data);
        data
    };
    let (riscv_gas, _) = call_contract(
        &mut sandbox,
        router_riscv.clone(),
        data_riscv,
        Some(BOB.clone()),
    );
    let (wasm_gas, _) = call_contract(&mut sandbox, router_wasm.clone(), data_wasm, Some(BOB));
    report_gas(riscv_gas, wasm_gas, "router", "swap_exact_tokens");
}

fn psp22_balance_of(
    sandbox: &mut MinimalSandbox,
    pair: AccountId32,
    caller: AccountId32,
) -> (u64, u128) {
    let data = {
        let mut data = vec![101u8, 104u8, 56u8, 47u8];
        caller.encode_to(&mut data);
        data
    };
    let (gas, balance) = call_contract(sandbox, pair, data, Some(caller));
    let balance = <Result<u128, ()>>::decode(&mut balance.as_slice())
        .unwrap()
        .unwrap();
    (gas, balance)
}

fn upload_contracts(sandbox: &mut MinimalSandbox) {
    for bytes in [
        psp22_riscv_bytes(),
        psp22_wasm_bytes(),
        wrapped_riscv_bytes(),
        wrapped_wasm_bytes(),
        factory_riscv_bytes(),
        factory_wasm_bytes(),
        pair_riscv_bytes(),
        pair_wasm_bytes(),
        router_riscv_bytes(),
        router_wasm_bytes(),
    ] {
        sandbox
            .upload_contract(
                bytes,
                MinimalSandbox::default_actor(),
                None,
                Determinism::Enforced,
            )
            .unwrap();
    }
}

fn factory_setup(
    sandbox: &mut MinimalSandbox,
    fee_to_setter: AccountId32,
) -> (AccountId32, AccountId32) {
    let data_riscv = {
        let mut data = NEW_SELECTOR.to_vec();
        fee_to_setter.encode_to(&mut data);
        BlakeTwo256::hash(&pair_riscv_bytes())
            .0
            .encode_to(&mut data);
        data
    };
    let data_wasm = {
        let mut data = NEW_SELECTOR.to_vec();
        fee_to_setter.encode_to(&mut data);
        BlakeTwo256::hash(&pair_wasm_bytes()).0.encode_to(&mut data);
        data
    };

    (
        instantiate_contract(
            sandbox,
            data_riscv,
            BlakeTwo256::hash(&factory_riscv_bytes()),
            None,
        ),
        instantiate_contract(
            sandbox,
            data_wasm,
            BlakeTwo256::hash(&factory_wasm_bytes()),
            None,
        ),
    )
}

fn psp22_setup(
    sandbox: &mut MinimalSandbox,
    name: String,
    caller: AccountId32,
) -> (AccountId32, AccountId32) {
    let data = {
        let mut data = NEW_SELECTOR.to_vec();
        PSP22_SUPPLY.encode_to(&mut data);
        Some(name.clone()).encode_to(&mut data);
        Some(name).encode_to(&mut data);
        PSP22_DECIMALS.encode_to(&mut data);
        data
    };

    (
        instantiate_contract(
            sandbox,
            data.clone(),
            BlakeTwo256::hash(&psp22_riscv_bytes()),
            Some(caller.clone()),
        ),
        instantiate_contract(
            sandbox,
            data,
            BlakeTwo256::hash(&psp22_wasm_bytes()),
            Some(caller),
        ),
    )
}

fn wazero_setup(sandbox: &mut MinimalSandbox) -> (AccountId32, AccountId32) {
    let data = NEW_SELECTOR.to_vec();

    (
        instantiate_contract(
            sandbox,
            data.clone(),
            BlakeTwo256::hash(&wrapped_riscv_bytes()),
            None,
        ),
        instantiate_contract(
            sandbox,
            data,
            BlakeTwo256::hash(&wrapped_wasm_bytes()),
            None,
        ),
    )
}

fn router_setup(
    sandbox: &mut MinimalSandbox,
    factory: (AccountId32, AccountId32),
    wazero: (AccountId32, AccountId32),
) -> (AccountId32, AccountId32) {
    let riscv_data = {
        let mut data = NEW_SELECTOR.to_vec();
        factory.0.encode_to(&mut data);
        wazero.0.encode_to(&mut data);
        data
    };
    let wasm_data = {
        let mut data = NEW_SELECTOR.to_vec();
        factory.1.encode_to(&mut data);
        wazero.1.encode_to(&mut data);
        data
    };

    (
        instantiate_contract(
            sandbox,
            riscv_data,
            BlakeTwo256::hash(&router_riscv_bytes()),
            None,
        ),
        instantiate_contract(
            sandbox,
            wasm_data,
            BlakeTwo256::hash(&router_wasm_bytes()),
            None,
        ),
    )
}

fn psp22_increase_allowance(
    sandbox: &mut MinimalSandbox,
    psp22: &AccountId32,
    router: &AccountId32,
    allowance: u128,
    caller: AccountId32,
) -> u64 {
    let data = {
        let mut data = vec![150u8, 214u8, 181u8, 122u8];
        router.encode_to(&mut data);
        allowance.encode_to(&mut data);
        data
    };

    call_contract(sandbox, psp22.clone(), data, Some(caller)).0
}

fn router_add_liquidity(
    sandbox: &mut MinimalSandbox,
    router: (&AccountId32, &AccountId32),
    ice: (&AccountId32, &AccountId32),
    wood: (&AccountId32, &AccountId32),
    desired_token_amount: u128,
    min_token_amount: u128,
    caller: AccountId32,
    deadline: u64,
) -> u128 {
    let data_riscv = {
        let mut data = vec![165u8, 183u8, 213u8, 151u8];
        ice.0.encode_to(&mut data);
        wood.0.encode_to(&mut data);
        desired_token_amount.encode_to(&mut data);
        desired_token_amount.encode_to(&mut data);
        min_token_amount.encode_to(&mut data);
        min_token_amount.encode_to(&mut data);
        caller.encode_to(&mut data);
        deadline.encode_to(&mut data);
        data
    };
    let data_wasm = {
        let mut data = vec![165u8, 183u8, 213u8, 151u8];
        ice.1.encode_to(&mut data);
        wood.1.encode_to(&mut data);
        desired_token_amount.encode_to(&mut data);
        desired_token_amount.encode_to(&mut data);
        min_token_amount.encode_to(&mut data);
        min_token_amount.encode_to(&mut data);
        caller.encode_to(&mut data);
        deadline.encode_to(&mut data);
        data
    };

    let (riscv_gas, riscv_return) =
        call_contract(sandbox, router.0.clone(), data_riscv, Some(caller.clone()));
    let (wasm_gas, wasm_return) = call_contract(sandbox, router.1.clone(), data_wasm, Some(caller));
    report_gas(riscv_gas, wasm_gas, "router", "add_liquidity");

    let liquidity_minted_riscv =
        <Result<Result<(u128, u128, u128), ()>, ()>>::decode(&mut riscv_return.as_slice())
            .unwrap()
            .unwrap()
            .unwrap();
    let liquidity_minted_wasm =
        <Result<Result<(u128, u128, u128), ()>, ()>>::decode(&mut wasm_return.as_slice())
            .unwrap()
            .unwrap()
            .unwrap();

    assert_eq!(liquidity_minted_riscv, liquidity_minted_wasm);
    liquidity_minted_riscv.2
}
