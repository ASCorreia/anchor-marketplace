use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TransferChecked, transfer_checked, TokenInterface};
use crate::{state::Marketplace, state::Listing};

#[derive(Accounts)]
pub struct Delist<'info> {
    #[account(mut)]
    maker: Signer<'info>,
    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump
    )]
    marketplace: Account<'info, Marketplace>,
    maker_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::authority = maker,
        associated_token::mint = maker_mint
    )]
    maker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"vault", maker_mint.key().as_ref()],
        bump = listing.vault_bump,
        token::authority = vault,
        token::mint = maker_mint
    )]
    vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        close = maker,
        seeds = [marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump = listing.bump
    )]
    listing: Account<'info, Listing>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>
}

impl<'info> Delist<'info> {
    pub fn withdraw_nft(&self) -> Result<()> {
        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker_ata.to_account_info(),
            authority: self.vault.to_account_info(),
            mint: self.maker_mint.to_account_info()
        };

        let seeds = &[
            &b"vault"[..], 
            &self.maker_mint.key().to_bytes()[..], 
            &[self.listing.vault_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds
        );

        transfer_checked(ctx, 1, self.maker_mint.decimals)
    }
}
