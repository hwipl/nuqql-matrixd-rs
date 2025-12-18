use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const ACCOUNTS_FILE: &str = "accounts.json";

#[derive(Clone, Deserialize, Serialize)]
pub struct Account {
    pub id: u32,
    pub protocol: String,
    pub user: String,
    pub password: String,
}

impl Account {
    fn new(id: u32, protocol: String, user: String, password: String) -> Self {
        Account {
            id: id,
            protocol: protocol,
            user: user,
            password: password,
        }
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
        self.accounts.values().cloned().collect()
    }

    pub async fn save(&self, file: &str) -> Result<(), Box<dyn std::error::Error>> {
        let accounts = self.list();
        let j = serde_json::to_vec(&accounts)?;
        let mut file = File::create(file).await?;
        file.write_all(&j).await?;
        Ok(())
    }

    pub async fn load(&mut self, file: &str) -> Result<(), Box<dyn std::error::Error>> {
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
