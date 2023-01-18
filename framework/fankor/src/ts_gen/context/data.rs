use crate::ts_gen::accounts::TsInstructionAccountGen;
use crate::ts_gen::types::{TsTypeGen, TsTypesCache};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// Contains the info for building the IDL.
pub struct DataContext {
    pub program_name: &'static str,
    pub accounts: HashSet<Cow<'static, str>>,
    pub account_types: TsTypesCache,
    pub account_schemas: TsTypesCache,
    pub program_methods: HashMap<&'static str, Cow<'static, str>>,

    // Type-value pairs.
    pub constants: HashMap<&'static str, (Cow<'static, str>, Cow<'static, str>)>,
}

impl DataContext {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new IDL build context.
    pub fn new() -> DataContext {
        DataContext {
            program_name: "program",
            accounts: HashSet::new(),
            account_types: TsTypesCache::new(),
            account_schemas: TsTypesCache::new(),
            program_methods: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Adds an account.
    pub fn set_context_name(&mut self, name: &'static str) -> Result<(), String> {
        self.program_name = name;

        Ok(())
    }

    /// Adds an account.
    pub fn add_account<T: TsTypeGen>(&mut self) -> Result<(), String> {
        let name = T::value_type();

        if self.accounts.contains(&name) {
            return Err(format!("Duplicated account name: '{}'", name));
        }

        T::generate_type(&mut self.account_types);
        T::generate_schema(&mut self.account_schemas);

        Ok(())
    }

    /// Adds a constant.
    pub fn add_constant<T: TsTypeGen>(
        &mut self,
        name: &'static str,
        value: T,
    ) -> Result<(), String> {
        if self.constants.contains_key(&name) {
            return Err(format!("Duplicated constant name: '{}'", name));
        }

        self.constants
            .insert(name, (T::value_type(), value.value()));

        Ok(())
    }

    /// Adds an instruction account.
    pub fn add_instruction_account<T: TsInstructionAccountGen>(&mut self) -> Result<(), String> {
        let name = T::value_type();

        if self.accounts.contains(&name) {
            return Err(format!("Duplicated instruction account: '{}'", name));
        }

        T::generate_type(&mut self.account_types);

        Ok(())
    }

    /// Adds a program method.
    pub fn add_program_method<T: TsInstructionAccountGen>(
        &mut self,
        name: &'static str,
        discriminant: u8,
    ) -> Result<(), String> {
        if self.program_methods.contains_key(name) {
            return Err(format!("Duplicated program method: '{}'", name));
        }

        let method = format!(
            "export function {}(accounts: {}) {{
                const data = Buffer.from([{}]);
                const accountMetas: solana.AccountMeta[] = [];

                {}

                return new solana.TransactionInstruction({{
                    keys: accountMetas,
                    programId: ID,
                    data
                }});
            }}",
            name,
            T::value_type(),
            discriminant,
            T::get_account_metas(Cow::Borrowed("accounts"), false, false),
        );

        self.program_methods.insert(name, Cow::Owned(method));

        Ok(())
    }

    /// Adds a program method.
    pub fn add_program_method_with_args<T: TsInstructionAccountGen, A: TsTypeGen>(
        &mut self,
        name: &'static str,
        discriminant: u8,
    ) -> Result<(), String> {
        if self.program_methods.contains_key(name) {
            return Err(format!("Duplicated program method: '{}'", name));
        }

        let method = format!(
            "export function {}(accounts: {}, args: {}) {{
                const argsBuffer = args.serialize();
                const data = Buffer.concat([Buffer.from([{}]), argsBuffer]);
                const accountMetas: solana.AccountMeta[] = [];

                {}

                return new solana.TransactionInstruction({{
                    keys: accountMetas,
                    programId: ID,
                    data
                }});
            }}",
            name,
            T::value_type(),
            A::value_type(),
            discriminant,
            T::get_account_metas(Cow::Borrowed("accounts"), false, false),
        );

        self.program_methods.insert(name, Cow::Owned(method));

        Ok(())
    }

    /// Builds the TypeScript file from the data stored in the context.
    pub fn build_ts_file(&mut self) -> String {
        let mut buffer = String::new();

        // Imports.
        buffer.push_str("import * as solana from '@solana/web3.js';");
        buffer.push_str("import * as fnk from '@suri-project/fankor/dist/esm';");
        buffer.push_str("import BN from 'bn.js';");

        // Build constants part.
        for (name, (ty, value)) in self.constants.iter() {
            buffer.push_str(format!("export const {}: {} = {};\n", name, ty, value).as_str());
        }

        // Build types.
        for (_name, type_definition) in self.account_types.iter() {
            buffer.push_str(&type_definition);
        }

        // Build schemas.
        for (_name, schema) in self.account_schemas.iter() {
            buffer.push_str(&schema);
        }

        // Build program methods.
        for (_name, method) in self.program_methods.iter() {
            buffer.push_str(&method);
        }

        buffer
    }
}
