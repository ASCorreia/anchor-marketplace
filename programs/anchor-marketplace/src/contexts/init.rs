use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::state::Marketplace;

#[derive(Accounts)]
#[instruction(name: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Marketplace::INIT_SPACE,
        seeds = [b"marketplace", name.as_str().as_bytes()],
        bump
    )]
    marketplace: Account<'info, Marketplace>,
    #[account(
        init,
        seeds = [b"rewards", marketplace.key().as_ref()],
        bump,
        payer = admin,
        mint::decimals = 6,
        mint::authority = marketplace,
    )]
    rewards: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump
    )]
    treasury: SystemAccount<'info>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init(&mut self, name: String, fee: u16, bumps: &InitializeBumps) -> Result<()> {
        match name.len() {
            0..=32 => {
                self.marketplace.set_inner( Marketplace {
                    admin: self.admin.key(),
                    fee,
                    name,
                    bump: bumps.marketplace,
                    treasury_bump: bumps.treasury,
                    rewards_bump: bumps.rewards,
                })
            }

            _ => msg!("Name too long"),
        }
        
        Ok(())
    }
}
