use crate::datastore::{DataStore, Op};

pub fn is_auth_required(ds: &mut DataStore) -> bool {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let store = ds.clone();
            let session = store.get("Authenticate".to_string());
            match session {
                Some(_) => true,
                None => false,
            }
        })
}

pub fn allowed_user(password: String, ds: &mut DataStore) -> bool {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let store = ds.clone();
            let user = store.get(password);
            match user {
                Some(_) => true,
                None => false,
            }
        })
}

pub fn allowed_session(key: String, ds: &mut DataStore) -> bool {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let store = ds.clone();
            let session = store.get(key);
            match session {
                Some(_) => true,
                None => false,
            }
        })
}
