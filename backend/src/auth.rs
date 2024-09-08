use crate::datastore::DataStore;

pub async fn auth_chain(key: String, ds: &mut DataStore) -> bool {
    if !is_auth_required(ds).await {
        return true;
    }
    if allowed_session(key, ds).await {
        return true;
    } 
    
    return false;
    
}


pub async fn is_auth_required(ds: &mut DataStore) -> bool {
    match ds.get("Authenticate".to_string()) {
        Some(a) => a == "true",
        None => false,
    }
}

pub async fn allowed_user(password: String, ds: &mut DataStore) -> bool {
    match ds.get("Password".to_string()) {
        Some(p) => p == password,
        None => false,
    }
}

pub async fn allowed_session(key: String, ds: &mut DataStore) -> bool {
    match ds.get("session".to_string()) {
        Some(s) => s == key,
        None => false,
    }
}
