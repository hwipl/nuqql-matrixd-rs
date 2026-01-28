use crate::matrix::Client;
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::error;

pub const ACCOUNTS_FILE: &str = "accounts.json";
const ACCOUNTS_FILE_PERMISSIONS: u32 = 0o600;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Account {
    pub id: u32,
    pub protocol: String,
    pub user: String,
    pub password: String,
    pub db_passphrase: String,
}

impl Account {
    fn new(id: u32, protocol: String, user: String, password: String) -> Self {
        Account {
            id: id,
            protocol: protocol,
            user: user,
            password: password,
            db_passphrase: Alphanumeric.sample_string(&mut rand::rng(), 16),
        }
    }

    /// Splits the user into user name and server.
    fn split_user(&self) -> (String, String) {
        match self.user.split_once("@") {
            Some(("", server)) => (self.user.clone(), server.into()),
            Some((user, "")) => (user.into(), self.user.clone()),
            Some((user, server)) => (user.into(), server.into()),
            None => (self.user.clone(), self.user.clone()),
        }
    }

    /// Gets the name of the account.
    pub fn get_name(&self) -> String {
        let (_, server) = self.split_user();
        format!("({server})")
    }

    pub fn get_status(&self) -> String {
        // TODO: get real status
        "offline".into()
    }

    pub fn start(&self) {
        let (user, server) = self.split_user();
        let client = Client::new(&server, &user, &self.password, &self.db_passphrase);
        tokio::spawn(async move {
            if let Err(err) = client.start().await {
                error!(user, server, error = %err, "Could not start matrix client")
            }
        });
    }
}

pub struct Accounts {
    accounts: HashMap<u32, Account>,
}

impl Accounts {
    pub fn new() -> Self {
        Accounts {
            accounts: HashMap::new(),
        }
    }

    fn get_free_account_id(&self) -> u32 {
        for id in 0..u32::MAX {
            if !self.accounts.contains_key(&id) {
                return id;
            }
        }
        u32::MAX
    }

    pub fn add(&mut self, protocol: String, user: String, password: String) {
        let id = self.get_free_account_id();
        let account = Account::new(id, protocol, user, password);
        self.accounts.insert(id, account);
    }

    pub fn remove(&mut self, id: &u32) {
        self.accounts.remove(id);
    }

    pub fn list(&self) -> Vec<Account> {
        let mut list: Vec<Account> = self.accounts.values().cloned().collect();
        list.sort_by_key(|x| x.id);
        return list;
    }

    pub async fn save(&self, file: &str) -> anyhow::Result<()> {
        let accounts = self.list();
        let j = serde_json::to_vec(&accounts)?;
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(ACCOUNTS_FILE_PERMISSIONS)
            .open(file)
            .await?;
        file.write_all(&j).await?;
        Ok(())
    }

    pub async fn load(&mut self, file: &str) -> anyhow::Result<()> {
        let mut file = File::open(file).await?;
        let mut j = vec![];
        file.read_to_end(&mut j).await?;
        let accounts: Vec<Account> = serde_json::from_slice(&j)?;
        for account in accounts {
            self.accounts.insert(account.id, account);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_free_account_id() {
        let mut accounts = Accounts::new();

        // empty
        assert_eq!(accounts.get_free_account_id(), 0);

        // add accounts
        accounts.add(
            "matrix".into(),
            "test-user1".into(),
            "test-password1".into(),
        );
        // -> accounts: 0
        assert_eq!(accounts.get_free_account_id(), 1);
        accounts.add(
            "matrix".into(),
            "test-user2".into(),
            "test-password2".into(),
        );
        // -> accounts: 0, 1
        assert_eq!(accounts.get_free_account_id(), 2);
        accounts.add(
            "matrix".into(),
            "test-user3".into(),
            "test-password3".into(),
        );
        // -> accounts: 0, 1, 2
        assert_eq!(accounts.get_free_account_id(), 3);

        // remove first and middle account
        accounts.remove(&0);
        // -> accounts: 1, 2
        assert_eq!(accounts.get_free_account_id(), 0);
        accounts.remove(&1);
        // -> accounts: 2
        assert_eq!(accounts.get_free_account_id(), 0);

        // add accounts again
        accounts.add(
            "matrix".into(),
            "test-user1".into(),
            "test-password1".into(),
        );
        // -> accounts: 0, 2
        assert_eq!(accounts.get_free_account_id(), 1);
        accounts.add(
            "matrix".into(),
            "test-user2".into(),
            "test-password2".into(),
        );
        // -> accounts: 0, 1, 2
        assert_eq!(accounts.get_free_account_id(), 3);

        // remove middle and first accounts
        accounts.remove(&1);
        // -> accounts: 0, 2
        assert_eq!(accounts.get_free_account_id(), 1);
        accounts.remove(&0);
        // -> accounts: 2
        assert_eq!(accounts.get_free_account_id(), 0);
    }

    #[tokio::test]
    async fn test_accounts_save_load() {
        // create temporary dir for accounts file
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("accounts.json");
        let file = file_path.to_str().unwrap();

        // load not existing
        let mut accounts = Accounts::new();
        accounts.load(file).await.unwrap_err();

        // save empty accounts, reset accounts, load empty
        let accounts = Accounts::new();
        let list = accounts.list();
        accounts.save(file).await.unwrap();

        let mut accounts = Accounts::new();
        accounts.load(file).await.unwrap();
        assert_eq!(accounts.list(), list);

        // add accounts, save accounts, reset accounts, load accounts
        let mut accounts = Accounts::new();
        accounts.add(
            "matrix".into(),
            "test-user1".into(),
            "test-password1".into(),
        );
        accounts.add(
            "matrix".into(),
            "test-user2".into(),
            "test-password2".into(),
        );
        let list = accounts.list();
        accounts.save(file).await.unwrap();
        let mut accounts = Accounts::new();
        accounts.load(file).await.unwrap();
        assert_eq!(accounts.list(), list);
    }
}
