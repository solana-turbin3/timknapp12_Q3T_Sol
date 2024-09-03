use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::state::{Game, PlayerEntry};

#[derive(Accounts)]
pub struct AddPlayer<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(mut,
        seeds = [b"game", game.admin.as_ref()],
        bump
    )]
    pub game: Account<'info, Game>,
    #[account(
        seeds = [b"vault", game.admin.as_ref(), game.key().as_ref(), game.game_code.as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddPlayer<'info> {
    pub fn add_player(&mut self) -> Result<()> {
        // Transfer entry fee
        let accounts = Transfer {
            from: self.player.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(cpi_ctx, self.game.entry_fee)?;

        // Check if player is already registered
        match self
            .game
            .players
            .iter()
            .find(|entry| entry.player == self.player.key())
        {
            None => {
                // Reallocate space for new player
                self.realloc(PlayerEntry::INIT_SPACE, &self.player, &self.system_program)?;

                // Add player to the game
                self.game.players.push(PlayerEntry {
                    player: self.player.key(),
                    num_correct: 0,
                    player_end_time: 0,
                });
                msg!("Player successfully added to the game");
                Ok(())
            }
            Some(_) => {
                msg!("Player already registered to the game");
                Ok(())
            }
        }
    }

    pub fn realloc(
        &self,
        space_to_add: usize,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        msg!("Reallocating account size to add player to the game");
        let account_info = self.game.to_account_info();
        let new_account_size = account_info.data_len() + space_to_add;

        // Determine additional rent required
        let lamports_required = (Rent::get()?).minimum_balance(new_account_size);
        let additional_rent_to_fund = lamports_required.saturating_sub(account_info.lamports());

        msg!(
            "Adding a new player has the cost of {:?} SOL",
            additional_rent_to_fund
        );

        // Perform transfer of additional rent
        let cpi_program = system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: payer.to_account_info(),
            to: account_info.clone(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_context, additional_rent_to_fund)?;

        // Reallocate the space on GAME account
        account_info.realloc(new_account_size, false)?;
        msg!("Account Size Updated");

        Ok(())
    }
}
