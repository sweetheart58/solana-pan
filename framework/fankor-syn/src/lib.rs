use syn::Error;

pub mod expressions;
pub mod solana;

pub type Result<T> = std::result::Result<T, Error>;
