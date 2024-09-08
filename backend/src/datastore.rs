use std::{
    collections::HashMap,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
        {Arc, Mutex},
    },
    thread,
};
#[derive(Debug, Clone)]
pub enum Op {
    Upsert(String, String),
    Remove(String),
}

#[derive(Debug, Clone)]
pub struct DataStore {
    store: Arc<Mutex<HashMap<String, String>>>,
    pub sender: Arc<Mutex<Sender<Op>>>,
    pub receiver: Arc<Mutex<Receiver<Op>>>,
}

impl AsRef<DataStore> for DataStore {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl DataStore {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        DataStore {
            store: Arc::new(Mutex::new(HashMap::new())),
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn all(&self) -> HashMap<String, String> {
        let locked_store = self.store.lock().unwrap();
        locked_store.clone()
    }

    pub fn get(&self, key: String) -> Option<String> {
        let locked_store = self.store.lock().unwrap();
        match locked_store.get(&key) {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    pub fn listen(&mut self) {
        let receiver = self.receiver.clone();
        let store = self.store.clone();
        thread::spawn(move || loop {
            let locked_receiver = receiver.lock().unwrap();
            let packet = match locked_receiver.recv() {
                Ok(msg) => msg,
                Err(e) => return eprintln!("{}", e),
            };

            let mut locked_store = store.lock().unwrap();
            match packet {
                Op::Upsert(key, value) => {
                    locked_store.insert(key, value);
                }
                Op::Remove(key) => {
                    locked_store.remove(&key);
                }
            }
        });
    }

    pub fn send(&self, msg: Op) {
        let sender = self.sender.lock().unwrap();
        sender.send(msg).unwrap();
    }
}
