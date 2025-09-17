#[cfg(test)]
pub mod cpi_guard;
#[cfg(test)]
pub mod group_member_pointer;
#[cfg(test)]
pub mod group_pointer;
#[cfg(test)]
pub mod initialize_mint;
#[cfg(test)]
pub mod scaled_ui_amount;
#[cfg(test)]
pub mod token_group;
#[cfg(test)]
pub mod token_group_member;

pub mod helpers {
    pub mod extensions {
        pub mod token_2022 {
            pub mod cpi_guard;
            pub mod group_member_pointer;
            pub mod group_pointer;
            pub mod initialize_mint;
            pub mod initialize_multisig;
            pub mod scaled_ui_amount;
            pub mod token_account;
            pub mod token_group;
        }
    }

    pub mod suite {
        pub mod core;
        pub mod solana_kite;
        pub mod types;
    }
}
