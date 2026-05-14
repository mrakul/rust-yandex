use std::sync::Arc;

use tracing::instrument;

use crate::data::account_repository::AccountRepository;
use crate::domain::{Account, Amount, BankError, DomainError, Transfer};

#[derive(Clone)]
pub struct BankService<R: AccountRepository + 'static> {
    repo: Arc<R>,
}

impl<R> BankService<R>
where
    R: AccountRepository + 'static,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn create_account(
        &self,
        id: u32,
        initial_balance: i64,
    ) -> Result<Account, BankError> {
        let account = Account::new(id, initial_balance).map_err(BankError::from)?;
        self.repo.create(account.clone()).await.map_err(|e| match e {
            DomainError::InvalidAmount(msg) => BankError::Validation(msg),
            other => BankError::from(other),
        })?;
        Ok(account)
    }

    #[instrument(skip(self))]
    pub async fn get_account(&self, id: u32) -> Result<Account, BankError> {
        match self.repo.get(id).await.map_err(BankError::from)? {
            Some(acc) => Ok(acc),
            None => Err(BankError::NotFound(format!("account {}", id))),
        }
    }

    #[instrument(skip(self))]
    pub async fn deposit(&self, id: u32, raw_amount: i64) -> Result<Account, BankError> {
        let mut account = self.get_account(id).await?;
        let amount = Amount::new(raw_amount).map_err(BankError::from)?;
        account.deposit(amount);
        self.repo.upsert(account.clone()).await.map_err(BankError::from)?;
        Ok(account)
    }

    #[instrument(skip(self))]
    pub async fn withdraw(&self, id: u32, raw_amount: i64) -> Result<Account, BankError> {
        let mut account = self.get_account(id).await?;
        let amount = Amount::new(raw_amount).map_err(BankError::from)?;
        account
            .withdraw(amount)
            .map_err(|err| match err {
                DomainError::InsufficientFunds => BankError::InsufficientFunds(id),
                other => BankError::from(other),
            })?;
        self.repo.upsert(account.clone()).await.map_err(BankError::from)?;
        Ok(account)
    }

    #[instrument(skip(self))]
    pub async fn transfer(&self, from: u32, to: u32, raw_amount: i64) -> Result<(), BankError> {
        let transfer = Transfer::new(from, to, raw_amount).map_err(BankError::from)?;

        let mut from_account = self.get_account(transfer.from).await?;
        let mut to_account = self.get_account(transfer.to).await?;

        from_account
            .withdraw(transfer.amount)
            .map_err(|err| match err {
                DomainError::InsufficientFunds => BankError::InsufficientFunds(transfer.from),
                other => BankError::from(other),
            })?;
        to_account.deposit(transfer.amount);

        self.repo.upsert(from_account).await.map_err(BankError::from)?;
        self.repo.upsert(to_account).await.map_err(BankError::from)?;

        Ok(())
    }
}


