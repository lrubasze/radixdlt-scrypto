use clap::Parser;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::resim::*;

/// Transfer resource to another account
#[derive(Parser, Debug)]
pub struct Transfer {
    /// The resource to transfer, e.g. "amount,resource_address" or "#nft_id1,#nft_id2,resource_address"
    resource: Resource,

    /// The recipient address
    recipient: Address,

    /// The transaction signers
    #[clap(short, long)]
    signers: Option<Vec<Address>>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Transfer {
    pub fn run(&self) -> Result<(), Error> {
        let mut runner = TransactionRunner::new()?;
        let default_account = runner.default_account()?;
        let default_signers = runner.default_signers()?;
        let transaction = TransactionBuilder::new(&runner.executor(self.trace))
            .withdraw_from_account(&self.resource, default_account)
            .call_method_with_all_resources(self.recipient, "deposit_batch")
            .build(self.signers.clone().unwrap_or(default_signers))
            .map_err(Error::TransactionConstructionError)?;
        runner.run_transaction(transaction, self.trace, |receipt| println!("{:?}", receipt))
    }
}
