use std::collections::HashMap;

struct Account {
    id: u32,
    protocol: String,
    user: String,
    password: String,
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
}
