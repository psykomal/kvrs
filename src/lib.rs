extern crate failure;
extern crate serde_json;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use failure::Error;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::{collections::HashMap, fs::File, path::PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

pub struct KvStore {
    index: HashMap<String, SizeInfo>,
    writer: BufWriter<File>,
    readers: Vec<BufReader<File>>,
}

/// File format:
/// start | size | key | value
///
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct SizeInfo {
    start: u64,
    segment_id: u32,
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
        fs::create_dir_all(&path)?;

        let mut filepaths = Vec::new();

        match fs::read_dir(path.clone()) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let file_path = entry.path();
                        filepaths.push(file_path);
                    }
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        filepaths.sort();

        if filepaths.len() == 0 {
            let fname = "kv_1.dat".to_string();

            let file_path = path.join(fname.clone());

            let _ = new_file(file_path.clone()).unwrap();

            filepaths.push(file_path.clone());
        }

        let readers = get_readers(&filepaths);
        let writer = get_current_writer(&filepaths);

        let mut kvstore = KvStore {
            index: HashMap::new(),
            writer,
            readers,
        };

        // println!("{:?}", filepaths);

        kvstore.build_index()?;

        return Ok(kvstore);
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(info) = self.index.get(&key) {
            let idx = info.segment_id - 1;
            let reader = &mut self.readers[idx as usize];
            reader.seek(SeekFrom::Start(info.start))?;
            let cmd_reader = reader.take(info.size);

            let KVPair { value, .. } = serde_json::from_reader(cmd_reader).unwrap();
            match value.as_str() {
                "rm" => Ok(None),
                _ => Ok(Some(value)),
            }
        } else {
            Ok(None)
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let pos = self.writer.seek(SeekFrom::End(0))?;
        let writer = &mut self.writer;
        let kv_pair = KVPair {
            key: key.clone(),
            value: value.clone(),
        };
        let kv_pair_serialized = serde_json::to_string(&kv_pair)?;

        writer.write_u64::<BigEndian>(kv_pair_serialized.len() as u64)?;
        write!(writer, "{}", kv_pair_serialized)?;

        self.writer.flush()?;
        self.index.insert(
            key.clone(),
            SizeInfo {
                start: pos + 8,
                segment_id: self.readers.len() as u32,
                size: kv_pair_serialized.len() as u64,
            },
        );
        Ok(())
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if let Some(_) = self.index.remove(&key) {
            self.set(key, "rm".to_string())
        } else {
            Err(failure::err_msg("Key not found"))
        }
    }

    fn build_index(&mut self) -> Result<()> {
        self.index = HashMap::new();

        for (i, reader) in self.readers.iter_mut().enumerate() {
            let mut pos = 0;
            let lastpos = reader.seek(SeekFrom::End(0))?;

            while pos < lastpos {
                reader.seek(SeekFrom::Start(pos))?;

                let sz = reader.read_u64::<BigEndian>().unwrap();

                reader.seek(SeekFrom::Start(pos + 8))?;
                let mut buf = vec![0; sz as usize];
                reader.read_exact(&mut buf)?;
                let kvpair: KVPair = serde_json::from_slice(&buf)?;

                self.index.insert(
                    kvpair.key.clone(),
                    SizeInfo {
                        start: pos + 8,
                        segment_id: (i + 1) as u32,
                        size: sz,
                    },
                );

                if kvpair.value == "rm".to_string() {
                    if let Some(_) = self.index.remove(&kvpair.key) {
                    } else {
                        return Err(failure::err_msg("Couldn't delete key"));
                    }
                }

                pos += 8 + sz;
            }
        }

        // println!("{:?}", self.index);

        Ok(())
    }
}

fn new_file(path: PathBuf) -> Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&path)?;

    Ok(file)
}

fn get_readers(filepaths: &Vec<PathBuf>) -> Vec<BufReader<File>> {
    let mut readers = Vec::new();

    for filepath in filepaths {
        let file = File::open(filepath).unwrap();
        readers.push(BufReader::new(file));
    }

    readers
}

fn get_current_writer(filepaths: &Vec<PathBuf>) -> BufWriter<File> {
    let len = filepaths.len();
    let file_path = filepaths[len - 1].clone();

    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&file_path)
        .unwrap();

    BufWriter::new(file)
}
