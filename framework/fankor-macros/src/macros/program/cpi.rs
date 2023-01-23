use crate::macros::program::programs::Program;
use crate::Result;
use convert_case::{Case, Converter};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn build_cpi(program: &Program) -> Result<TokenStream> {
    let case_converter = Converter::new()
        .from_case(Case::Snake)
        .to_case(Case::Pascal);

    let methods = program.methods.iter().map(|v| {
        let discriminant = format_ident!("{}", case_converter.convert(v.name.to_string()), span = v.name.span());
        let program_name = &program.name;
        let method_name = &v.name;
        let account_type = &v.account_type;
        let discriminant_name = format_ident!("{}Discriminant", program_name);

        let (arguments, argument_param) = if let Some(argument_type) = &v.argument_type {
            let arguments = quote! {
                let mut ix_data = ::fankor::prelude::borsh::BorshSerialize::try_to_vec(&arguments)?;
                data.append(&mut ix_data);
            };

            (arguments, quote! {
                , arguments: &#argument_type
            })
        } else {
            (quote! {}, quote! {})
        };

        let (result, result_param) = if let Some(result_type) = &v.result_type {
            (quote! {
                Ok(::fankor::models::CpiReturn::new())
            }, quote! {
                ::fankor::models::CpiReturn<#result_type>
            })
        } else {
            (quote! { Ok(()) }, quote! { () })
        };

        quote! {
            pub fn #method_name<'info>(_program: &::fankor::models::Program<super::#program_name>, accounts: <#account_type as ::fankor::traits::InstructionAccount<'info>>::CPI #argument_param, signer_seeds: &[&[&[u8]]]) -> ::fankor::errors::FankorResult<#result_param> {
                let mut data = vec![#discriminant_name::#discriminant.code()];
                #arguments

                let mut metas = Vec::new();
                let mut infos = Vec::new();
                ::fankor::traits::CpiInstructionAccount::to_account_metas_and_infos(&accounts, &mut metas, &mut infos)?;

                let instruction = ::fankor::prelude::solana_program::instruction::Instruction {
                    program_id: *<super::#program_name as ::fankor::traits::ProgramType>::address(),
                    accounts: metas,
                    data,
                };

                ::fankor::prelude::solana_program::program::invoke_signed(&instruction, &infos, signer_seeds)
                    .map_or_else(|e| Err(::fankor::errors::Error::ProgramError(e)), |_| Ok(()))?;

                #result
            }
        }
    });

    Ok(quote! {
        pub mod cpi {
            //! CPI methods for calling this program's instructions inside another Solana program.

            use super::*;

            #(#methods)*
        }
    })
}
