#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount};

#[program]
pub mod faucet {
    use anchor_spl::token::{MintTo, Transfer};

    use super::*;
    pub fn initialize(ctx: Context<Initialize>, nonce: u8) -> ProgramResult {
        let faucet = &mut ctx.accounts.faucet;

        faucet.mint = ctx.accounts.mint.key.clone();
        faucet.nonce = nonce;

        Ok(())
    }

    pub fn drip(ctx: Context<Drip>) -> ProgramResult {
        let seeds = &[
            ctx.accounts.faucet.to_account_info().key.as_ref(),
            &[ctx.accounts.faucet.nonce],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.receiver.to_account_info(),
            authority: ctx.accounts.mint_auth.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.clone();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts,signer);
        token::mint_to(cpi_ctx, 10)?;

        Ok(())
    }

    pub fn transfer(ctx: Context<Trans>, nonce: u8) -> ProgramResult {
        let seeds = &[
            ctx.accounts.faucet.to_account_info().key.as_ref(),
            &[nonce],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.clone();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts,signer);
        token::transfer(cpi_ctx, 10)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init)]
    faucet: ProgramAccount<'info, Faucet>,
    mint: AccountInfo<'info>,

    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Drip<'info> {
    #[account()]
    faucet: ProgramAccount<'info, Faucet>,

    #[account(mut)]
    mint: CpiAccount<'info, Mint>,

    // what's the point with this annotation?
    #[account(seeds = [faucet.to_account_info().key.as_ref(), &[faucet.nonce]])]
    mint_auth: AccountInfo<'info>,

    #[account(mut)]
    receiver: CpiAccount<'info, TokenAccount>,

    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Trans<'info> {
    #[account()]
    faucet: ProgramAccount<'info, Faucet>,

    #[account(mut)]
    from: CpiAccount<'info, TokenAccount>,

    // Owner of the `from` token account.
    owner: AccountInfo<'info>,

    #[account(seeds = [
        faucet.to_account_info().key.as_ref(),
        &[faucet.nonce],
    ])]
    check_signer: AccountInfo<'info>,

    #[account(mut)]
    to: CpiAccount<'info, TokenAccount>,

    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

// impl<'info> Trans<'info> {
//     pub fn accounts(ctx: &Context<Trans>, nonce: u8) -> Result<()> {
//         let signer = Pubkey::create_program_address(
//             &[ctx.accounts.check.to_account_info().key.as_ref(), &[nonce]],
//             ctx.program_id,
//         )
//         .map_err(|_| ErrorCode::InvalidCheckNonce)?;
//         if &signer != ctx.accounts.check_signer.to_account_info().key {
//             return Err(ErrorCode::InvalidCheckSigner.into());
//         }
//         Ok(())
//     }
// }

#[account]
pub struct Faucet {
    pub mint: Pubkey,
    // signer nonce
    pub nonce: u8,
}