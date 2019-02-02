// use fail::fail_point;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Mutex;
use tempdir::TempDir;

pub struct Account {
    name: String,
    data_dir: TempDir,
}

impl Account {
    pub fn new(name: impl Into<String>, remaining: u64) -> Account {
        let data_path = TempDir::new("test_account").unwrap();
        let mut account = Account {
            name: name.into(),
            data_dir: data_path,
        };
        account.set_remaining(remaining);
        account
    }

    pub fn remaining(&self) -> u64 {
        let data_file = self.data_dir.path().join(&self.name);
        let mut f = File::open(data_file).unwrap();
        let mut s = [0; 8];
        f.read_exact(&mut s).unwrap();
        u64::from_be_bytes(s)
    }

    pub fn set_remaining(&mut self, remaining: u64) {
        let data_file = self.data_dir.path().join(&self.name);
        let mut f = File::create(&data_file).unwrap();
        f.write_all(&u64::to_be_bytes(remaining)).unwrap();
    }
}

pub fn transfer(from: &Mutex<Account>, to: &Mutex<Account>, amount: u64) -> bool {
    let f_remaining = from.lock().unwrap().remaining();
    if f_remaining < amount {
        return false;
    }
    // fail_point!("slow_update");
    from.lock().unwrap().set_remaining(f_remaining - amount);
    let t_remaining = to.lock().unwrap().remaining();
    to.lock().unwrap().set_remaining(t_remaining + amount);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    // use fail;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_transfer() {
        // fail::setup();
        // fail::cfg("slow_update", "sleep(100)").unwrap();
        let ana = Arc::new(Mutex::new(Account::new("ana", 50)));
        let jay = Arc::new(Mutex::new(Account::new("jay", 50)));
        let mut handlers: Vec<_> = (0..5)
            .map(|_| {
                let (ana, jay) = (ana.clone(), jay.clone());
                thread::spawn(move || {
                    transfer(&ana, &jay, 10);
                })
            })
            .collect();
        handlers.drain(..).map(|h| h.join().unwrap()).count();
        let ana_remaining = ana.lock().unwrap().remaining();
        assert_eq!(ana_remaining, 0);
        let jay_remaining = jay.lock().unwrap().remaining();
        assert_eq!(jay_remaining, 100);
        // fail::teardown();
    }
}
