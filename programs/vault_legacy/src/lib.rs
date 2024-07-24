use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program::pubkey;
declare_id!("Bf3PTV4KDsG7S2WEAd1QwbM8F3yHT8ruPz8gAN4yE8e9");

// Data Logic
#[program]
pub mod vault {
    use anchor_spl::token::{burn, mint_to, Burn, MintTo};
    use solana_program::{
        program::{invoke, invoke_signed},
        system_instruction::transfer,
    };

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let authority = &mut ctx.accounts.authority;
        let vault = &mut ctx.accounts.vault;

        // Set the authority in the config (useful for gating who can call certain functions or update certain data in acounts in the future)
        let config = &mut ctx.accounts.config;
        config.authority = authority.key();

        // Tranfer 0.01 Sol to the vault to ghost init it
        invoke(
            &transfer(authority.key, &vault.key, 10000000),
            &[authority.to_account_info(), vault.to_account_info()],
        )?;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let authority = &mut ctx.accounts.authority;
        let config = &mut ctx.accounts.config;
        let mint = &mut ctx.accounts.mint;
        let mint_bump = ctx.bumps.mint;
        let vault = &mut ctx.accounts.vault;
        let shares = &mut ctx.accounts.shares;
        let token_program = &mut ctx.accounts.token_program;

        let current_user_shares = shares.amount;
        let shares_to_mint: u64;
        let vault_balance = vault.lamports() - 10000000;
        if vault_balance == 0 {
            shares_to_mint = amount
        } else {
            shares_to_mint = (amount * vault_balance) / current_user_shares
        }

        // Transfer amount to the vault
        invoke(
            &transfer(authority.key, &vault.key, amount),
            &[authority.to_account_info(), vault.to_account_info()],
        )?;

        // Mint shares
        mint_to(
            CpiContext::new(
                token_program.to_account_info(),
                MintTo {
                    mint: mint.to_account_info(),
                    to: shares.to_account_info(),
                    authority: authority.to_account_info(),
                },
            )
            .with_signer(&[&[config.key().as_ref(), &[mint_bump]]]),
            shares_to_mint,
        )?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let authority = &mut ctx.accounts.authority;
        let config = &mut ctx.accounts.config;
        let mint = &mut ctx.accounts.mint;
        let mint_bump = ctx.bumps.mint;
        let vault = &mut ctx.accounts.vault;
        let vault_bump = ctx.bumps.vault;
        let shares = &mut ctx.accounts.shares;
        let token_program = &mut ctx.accounts.token_program;

        let current_user_shares = shares.amount;
        let vault_balance = vault.lamports() - 10000000;

        let shares_to_withdraw = (amount * current_user_shares) / vault_balance;

        // Burn shares
        burn(
            CpiContext::new(
                token_program.to_account_info(),
                Burn {
                    mint: mint.to_account_info(),
                    from: shares.to_account_info(),
                    authority: authority.to_account_info(),
                },
            )
            .with_signer(&[&[config.key().as_ref(), &[mint_bump]]]),
            amount,
        )?;

        // Transfer amount from the vault to the user
        invoke_signed(
            &transfer(&vault.key, authority.key, shares_to_withdraw),
            &[authority.to_account_info(), vault.to_account_info()],
            &[&[b"vault", &[vault_bump]]],
        )?;

        Ok(())
    }
}

// Data Validators
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, seeds =[b"config"], payer = authority, bump, space = Config::LENGTH)]
    pub config: Account<'info, Config>,
    /// CHECK: vault is validated by its seed phrase
    #[account(seeds = [b"vault"], bump)]
    pub vault: UncheckedAccount<'info>,
    #[account(mut, address = pubkey!("CTgFKhrDBN61VJ91BzTdyoK1MkdBTwkJvdv1BKiDWask"))]
    pub authority: Signer<'info>,
    #[account(init, seeds = [config.key().as_ref()], bump, payer = authority, mint::decimals = 9, mint::authority = mint)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"config"], bump, has_one = authority)]
    pub config: Account<'info, Config>,
    /// CHECK: vault is validated by its seed phrase
    #[account(seeds = [b"vault"], bump)]
    pub vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init_if_needed, payer = authority, associated_token::mint = mint, associated_token::authority = authority)]
    pub shares: Account<'info, TokenAccount>,
    #[account(seeds = [config.key().as_ref()], bump)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"config"], bump, has_one = authority)]
    pub config: Account<'info, Config>,
    /// CHECK: vault is validated by its seed phrase
    #[account(seeds = [b"vault"], bump)]
    pub vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = authority)]
    pub shares: Account<'info, TokenAccount>,
    #[account(seeds = [config.key().as_ref()], bump)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Data Structures
#[account]
pub struct Config {
    pub authority: Pubkey,
}

impl Config {
    const LENGTH: usize = 8 + 32;
}

// Utils

// Errors
