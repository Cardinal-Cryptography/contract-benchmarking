# contract-benchmarking

Comparing riscv and wasm contract performance with drink.

# Reproduction

```bash
make pull-image
make run-flipper-simulation
make run-dex-simulation
```

Output should be:
```
----------------------------------------
[flipper][riscv][deploy]: 132,782,316
[flipper][wasm] [deploy]: 267,840,354
Speedup: 50%
----------------------------------------
[flipper][riscv][flip]: 255,099,016
[flipper][wasm] [flip]: 395,707,235
Speedup: 64%
----------------------------------------
[factory][riscv][set_fee_collector]: 613,937,156
[factory][wasm] [set_fee_collector]: 764,658,899
Speedup: 80%
----------------------------------------
[psp22][riscv][increase_allowance]: 1,406,080,363
[psp22][wasm] [increase_allowance]: 2,133,153,103
Speedup: 66%
----------------------------------------
[router][riscv][add_liquidity]: 27,091,657,839
[router][wasm] [add_liquidity]: 73,103,165,817
Speedup: 37%
----------------------------------------
[factory][riscv][get_pair]: 512,171,913
[factory][wasm] [get_pair]: 569,226,744
Speedup: 90%
----------------------------------------
[psp22][riscv][balance_of]: 2,152,606,040
[psp22][wasm] [balance_of]: 1,695,776,569
Speedup: 127%
----------------------------------------
[router][riscv][swap_exact_tokens]: 19,396,361,968
[router][wasm] [swap_exact_tokens]: 51,087,540,671
Speedup: 38%
```

# Notes

- You can edit [Dockerfile](Dockerfile) to use a different version of the toolchain.
Then build image with `make build-image`.
Remember also to remove old build files with `make clean`.
- You may want to play a bit with [`polkadot-sdk`](polkadot-sdk) versions.

# Problems

1. We always use `cargo contract`.
Running `cargo +rve-nightly contract build` results in errors like:
```
error[E0658]: use of unstable library feature 'stdsimd'
  --> /root/.cargo/registry/src/index.crates.io-6f17d22bba15001f/curve25519-dalek-4.1.2/src/backend/vector/ifma/field.rs:33:9
   |
33 |     use core::arch::x86_64::_mm256_madd52hi_epu64;
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: see issue #48556 <https://github.com/rust-lang/rust/issues/48556> for more information
   = help: add `#![feature(stdsimd)]` to the crate attributes to enable

error[E0635]: unknown feature `stdarch_x86_avx512`
  --> /root/.cargo/registry/src/index.crates.io-6f17d22bba15001f/curve25519-dalek-4.1.2/src/lib.rs:19:13
   |
19 |     feature(stdarch_x86_avx512)
   |      
...
```
2. We use `https://github.com/paritytech/rustc-rv32e-toolchain/releases/download/v1.0.0` instead of `https://github.com/paritytech/rustc-rv32e-toolchain/releases/download/v1.1.0`.
The newer version is failing even for flipper with:
```
ERROR: Failed to link polkavm program: found control instruction at the end of block at <section #9+760> whose target doesn't resolve to any basic block:
Call { ra: RA, target: SectionTarget { section_index: SectionIndex(22), offset: 0 }, target_return: SectionTarget { section_index: SectionIndex(9), offset: 780 } }
```