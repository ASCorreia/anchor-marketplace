use crate::{
    state::Marketplace, state::Listing,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{Metadata, MetadataAccount, MasterEditionAccount},
    token_interface::{TransferChecked, transfer_checked, Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct List<'info> {
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
        init,
        payer = maker,
        seeds = [b"vault", maker_mint.key().as_ref()],
        bump,
        token::authority = vault,
        token::mint = maker_mint
    )]
    vault: InterfaceAccount<'info, TokenAccount>,
    collection_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = maker,
        space = Listing::INIT_SPACE,
        seeds = [marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump
    )]
    listing: Account<'info, Listing>,
    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            maker_mint.key().as_ref()
        ],
        seeds::program = metadata_program.key(),
        bump,
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref()
    )]
    metadata: Account<'info, MetadataAccount>,
    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            maker_mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = metadata_program.key(),
        bump
    )]
    master_edition: Account<'info, MasterEditionAccount>,
    metadata_program: Program<'info, Metadata>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> List<'info> {
    pub fn create_listing(&mut self, price: u64, bumps: &ListBumps) -> Result<()> {
        self.listing.set_inner(Listing { 
            maker: self.maker.key(), 
            mint: self.maker_mint.key(), 
            price, 
            bump: bumps.listing, 
            vault_bump: bumps.vault 
        });

        Ok(())
    }

    pub fn deposit_nft(&self) -> Result<()> {
        let accounts = TransferChecked {
            from: self.maker_ata.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
            mint: self.maker_mint.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer_checked(ctx, 1, self.maker_mint.decimals)
    }
}
