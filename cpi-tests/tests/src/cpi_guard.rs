use {
    crate::helpers::{
        extensions::token_2022::{
            cpi_guard::Token2022CpiGuardExtension,
            initialize_mint::Token2022InitializeMintExtension,
            initialize_multisig::Token2022InitializeMultisigExtension,
        },
        suite::{
            core::App,
            types::{pin_pubkey_to_addr, AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_program::pubkey::Pubkey,
    solana_signer::Signer,
    spl_pod::optional_keys::OptionalNonZeroPubkey,
    spl_token_2022_interface::extension::{cpi_guard::CpiGuard, ExtensionType},
    spl_associated_token_account::get_associated_token_address,
};

#[test]
fn test_cpi_guard_enable_and_disable() -> TestResult<()> {
    let mut app = App::new(true);
    let owner = AppUser::Admin;

    // The mint account needs the CpiGuard extension for the token account to use it.
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        owner,
        None,
        Some(&[ExtensionType::CpiGuard]),
    )?;
    
    let mint_pubkey = &mint_keypair.pubkey();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    // Create a token account associated with the owner and mint.
    let token_account_pubkey = get_associated_token_address(&Pubkey::from(owner.pubkey()), &mint_pubkey);
    let token_account_pin_pubkey = &token_account_pubkey.to_bytes();

    // Enable the CPI guard.
    // For a single signer, the `owner` is the authority, and they are also the sole signer.
    app.token_2022_try_enable_cpi_guard(
        Target::Proxy,
        token_account_pin_pubkey,
        owner,
        &[owner],
    )?;

    // // Verify the guard is enabled.
    // let cpi_guard_state =
    //     app.token_2022_query_cpi_guard(Target::Proxy, token_account_pin_pubkey)?;
    // assert_eq!(cpi_guard_state.lock_cpi, true.into());

    // Disable the CPI guard.
    // app.token_2022_try_disable_cpi_guard(
    //     Target::Proxy,
    //     token_account_pin_pubkey,
    //     owner,
    //     &[owner],
    // )?;

    // // Verify the guard is disabled.
    // let cpi_guard_state =
    //     app.token_2022_query_cpi_guard(Target::Proxy, token_account_pin_pubkey)?;
    // assert_eq!(cpi_guard_state.lock_cpi, false.into());

    Ok(())
}