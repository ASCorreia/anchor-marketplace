use anchor_lang::error_code;

#[error_code]
pub enum MarketplaceError {
    #[msg("Name must be between 1 and 32 characters")]
    NameTooLong,
}