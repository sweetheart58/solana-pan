use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::{AccountInfoVerification, InstructionAccount, PdaChecker};
use solana_program::account_info::AccountInfo;

impl<'info, T: InstructionAccount<'info>> InstructionAccount<'info> for Option<T> {
    type CPI = Option<T::CPI>;
    type LPI = Option<T::LPI>;

    #[inline(always)]
    fn min_accounts() -> usize {
        0 // Because None does not require any accounts.
    }

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        match self {
            Some(account) => account.verify_account_infos(config),
            None => Ok(()),
        }
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        let mut new_accounts = *accounts;
        match T::try_from(context, &mut new_accounts) {
            Ok(v) => {
                *accounts = new_accounts;
                Ok(Some(v))
            }
            Err(_) => Ok(None),
        }
    }
}

impl<'info, T: PdaChecker<'info>> PdaChecker<'info> for Option<T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        match self {
            Some(v) => v.pda_info(),
            None => None,
        }
    }
}
