#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod riscy {
    #[ink(storage)]
    pub struct Riscy {
        value: bool,
    }

    impl Riscy {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }
}
