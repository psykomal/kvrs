use crate::{KvsEngine, Result};

pub struct SledKvsEngine {
    store: sled::Db,
}

impl SledKvsEngine {
    pub fn open(path: &str) -> Self {
        let store = sled::open(path).unwrap();

        Self { store }
    }
}

// impl KvsEngine for SledKvsEngine {
//     fn get(&mut self, key: String) -> Result<Option<String>> {
//         match self.store.get(key.as_bytes()) {
//             Ok(Some(value)) => Ok(Some(String::from_utf8(value.to_vec()).unwrap())),
//             Ok(None) => Ok(None),
//             Err(e) => Err(e.into()),
//         }
//     }

//     fn set(&mut self, key: String, value: String) -> Result<()> {
//         self.store.insert(key.as_bytes(), value.as_bytes())?;

//         self.store.flush()?;

//         Ok(())
//     }

//     fn remove(&mut self, key: String) -> Result<()> {
//         match self.store.remove(key.as_bytes()) {
//             Ok(Some(_)) => {
//                 self.store.flush()?;
//                 Ok(())
//             }
//             Ok(None) => Err(failure::err_msg("Key not found")),
//             Err(e) => Err(e.into()),
//         }
//     }
// }
