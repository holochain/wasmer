use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use wasmer::*;

use crate::{common::HashAlgorithm, store::StoreOptions};

#[derive(Debug, Parser)]
/// The options for the `wasmer validate` subcommand
pub struct Validate {
    /// File to validate as WebAssembly
    #[clap(name = "FILE")]
    path: PathBuf,

    #[clap(flatten)]
    store: StoreOptions,

    /// Hashing algorithm to be used for module hash
    #[clap(long, value_enum)]
    hash_algorithm: Option<HashAlgorithm>,
}

impl Validate {
    /// Runs logic for the `validate` subcommand
    pub fn execute(&self) -> Result<()> {
        self.inner_execute()
            .context(format!("failed to validate `{}`", self.path.display()))
    }
    fn inner_execute(&self) -> Result<()> {
        let (store, _compiler_type) = self.store.get_store()?;

        let mut engine = store.engine().clone();
        let hash_algorithm = self.hash_algorithm.unwrap_or_default().into();
        engine.set_hash_algorithm(Some(hash_algorithm));

        let module_contents = std::fs::read(&self.path)?;
        if !is_wasm(&module_contents) {
            bail!("`wasmer validate` only validates WebAssembly files");
        }
        Module::validate(&store, &module_contents)?;
        eprintln!("Validation passed for `{}`.", self.path.display());
        Ok(())
    }
}
