use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::{token_interface::{Mint, TokenAccount, TransferChecked, transfer_checked, CloseAccount, close_account, TokenInterface, MintTo, mint_to}, associated_token::AssociatedToken};
use crate::{state::Marketplace, state::Listing};

#[derive(Accounts)]
pub struct Purchase<'info> {
    #[account(mut)]
    taker: Signer<'info>,
    #[account(mut)]
    maker: SystemAccount<'info>,
    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump
    )]
    marketplace: Account<'info, Marketplace>,
    #[account(
        seeds = [b"rewards", marketplace.key().as_ref()],
        bump = marketplace.rewards_bump,
        mint::decimals = 6,
        mint::authority = marketplace,
    )]
    rewards: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = maker_mint,
        associated_token::authority = taker,
    )]
    taker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"vault", maker_mint.key().as_ref()],
        bump = listing.vault_bump,
        token::authority = vault,
        token::mint = maker_mint
    )]
    vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = rewards,
        associated_token::authority = taker,
    )]
    taker_rewards_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump = marketplace.treasury_bump
    )]
    treasury: SystemAccount<'info>,
    maker_mint: InterfaceAccount<'info, Mint>,
    collection_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        close = maker,
        has_one = maker,
        seeds = [marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump = listing.bump
    )]
    listing: Account<'info, Listing>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>
}

impl<'info> Purchase<'info> {
    pub fn send_sol(&self) -> Result<()> {
        let accounts = Transfer {
            from: self.taker.to_account_info(),
            to: self.maker.to_account_info()
        };
        let ctx = CpiContext::new(
            self.system_program.to_account_info(), 
            accounts
        );
        let amount = self.listing.price - self.marketplace.fee as u64;
        transfer(ctx, self.listing.price - amount)?;

        let accounts = Transfer {
            from: self.taker.to_account_info(),
            to: self.system_program.to_account_info()
        };
        let ctx = CpiContext::new(
            self.system_program.to_account_info(), 
            accounts
        );
        
        transfer(ctx, self.marketplace.fee as u64)
    }

    pub fn send_nft(&self) -> Result<()> {
        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.taker_ata.to_account_info(),
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

    pub fn mint_rewards(&self) -> Result<()> {
        let seeds = &[
            &b"marketplace"[..],
            &self.marketplace.key().to_bytes()[..],
            &[self.marketplace.rewards_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let accounts = MintTo {
            mint: self.rewards.to_account_info(),
            to: self.taker_rewards_ata.to_account_info(),
            authority: self.marketplace.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds
        );

        mint_to(ctx, 5_000_000)
    }

    pub fn close_mint_ata(&mut self) -> Result<()> {
        let accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.vault.to_account_info()
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

        close_account(ctx)
    }

}
