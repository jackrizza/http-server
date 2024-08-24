use crate::datastore::{DataStore, Op};

pub async fn is_auth_required(ds: &mut DataStore) -> bool {
    let store = ds.clone();
    let session = store.get("Authenticate".to_string());
    match session {
        Some(_) => true,
        None => false,
    }
}

pub async fn allowed_user(password: String, ds: &mut DataStore) -> bool {
    let store = ds.clone();
    let user = store.get("Password".to_string());
    match user {
        Some(p) => {
            if p == password {
                true
            } else {
                false
            }
        }
        None => false,
    }
}

pub async fn allowed_session(key: String, ds: &mut DataStore) -> bool {
    let store = ds.clone();
    let session = store.get("session".to_string());
    match session {
        Some(s) => {
            if s == key {
                true
            } else {
                false
            }
        }
        None => false,
    }
}
