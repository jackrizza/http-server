use crate::datastore::DataStore;
use actix_web::HttpRequest;

pub async fn auth_chain(req: HttpRequest, key: String, ds: &mut DataStore) -> bool {
    if !is_auth_required(ds).await {
        return true;
    }
    if allowed_session(req, key, ds).await {
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

pub async fn allowed_session(req: HttpRequest, key: String, ds: &mut DataStore) -> bool {
    let ip = match req.peer_addr() {
        Some(ip) => ip.ip().to_string(),
        None => "unknown".to_string(),
    };
    let id = match ds.get(ip.clone()) {
        Some(id) => id,
        None => return false,
    };

    let k = match ds.get(id.clone()) {
        Some(key) => key,
        None => return false,
    };

    println!(
        "ip : {}, id : {} key : {}, k : {}\nkey == k => {}",
        ip,
        id,
        key,
        k,
        key == k
    );

    k == key
}
