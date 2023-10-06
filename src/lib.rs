extern crate failure;
extern crate serde_json;

use failure::Error;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::{collections::HashMap, fs::File, path::PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

pub struct KvStore {
    index: HashMap<String, SizeInfo>,
    writer: BufWriter<File>,
    reader: BufReader<File>,
}

/// File format:
/// start | size | key | value
///
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct SizeInfo {
    start: u64,
    size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct KVPair {
    key: String,
    value: String,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();

        Ok(KvStore {
            index: HashMap::new(),
            writer: BufWriter::new(File::create(&path)?),
            reader: BufReader::new(File::open(&path)?),
        })
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        self.build_index();

        if let Some(info) = self.index.get(&key) {
            let reader = &mut self.reader;
            reader.seek(SeekFrom::Start(info.start))?;
            let cmd_reader = reader.take(info.size);

            if let KVPair { value, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(failure::err_msg("invalid value"))
            }
        } else {
            Ok(None)
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let pos = self.writer.seek(SeekFrom::End(0))?;
        let mut writer = &mut self.writer;
        let kv_pair = KVPair {
            key: key.clone(),
            value: value.clone(),
        };
        let kv_pair_serialized = serde_json::to_string(&kv_pair)?;

        let sizeinfo = SizeInfo {
            start: pos,
            size: kv_pair_serialized.len() as u64,
        };
        let sizeinfo_serialized = serde_json::to_string(&sizeinfo)?;

        write!(writer, "{}", sizeinfo_serialized)?;
        write!(writer, "{}", kv_pair_serialized)?;

        self.writer.flush()?;
        self.index.insert(
            key.clone(),
            SizeInfo {
                start: pos,
                size: self.writer.seek(SeekFrom::End(0))? - pos,
            },
        );
        Ok(())
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        self.build_index()?;

        if let Some(info) = self.index.remove(&key) {
            self.set(key, "rm".to_string())
        } else {
            Err(failure::err_msg("Key not found"))
        }
    }

    fn build_index(&mut self) -> Result<()> {
        let reader = &mut self.reader;
        self.index = HashMap::new();
        let mut pos = 0;
        let lastpos = self.writer.seek(SeekFrom::End(0))?;

        while pos < lastpos {
            reader.seek(SeekFrom::Start(pos))?;
            let sz_reader = reader.take(16);
            let szinfo: SizeInfo = serde_json::from_reader(sz_reader)?;

            reader.seek(SeekFrom::Start(pos + 16))?;
            let kv_reader = reader.take(szinfo.size);
            let kvpair: KVPair = serde_json::from_reader(kv_reader)?;

            self.index.insert(kvpair.key.clone(), szinfo);

            if kvpair.value == "rm".to_string() {
                if let Some(_) = self.index.remove(&kvpair.key) {
                } else {
                    return Err(failure::err_msg("Couldn't delete key"));
                }
            }

            pos += 16 + szinfo.size;
        }

        Ok(())
    }
}
