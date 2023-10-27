extern crate failure;
extern crate serde_json;

use crate::{KvsEngine, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, fs::File, path::PathBuf};

const INITIAL_MAX_SEGMENT_SIZE: u64 = 1024;
const NUM_SEGMENTS_COMPACTION_THREASHOLD: u32 = 4;

#[derive(Clone)]
pub struct KvStore {
    index: Arc<Mutex<HashMap<String, SizeInfo>>>,
    writer: Arc<Mutex<StoreWriter>>,
    reader: StoreReader,
    dir: PathBuf,
    rwmutex: Arc<std::sync::RwLock<()>>,
}

struct StoreReader {
    dir: PathBuf,
    readers: Vec<BufReader<File>>,
}

impl Clone for StoreReader {
    fn clone(&self) -> Self {
        StoreReader {
            dir: self.dir.clone(),
            readers: get_readers_dir(self.dir.clone()),
        }
    }
}

struct StoreWriter {
    dir: PathBuf,
    writer: BufWriter<File>,
    max_segment_size: u64,
    curr_gen: u32,
}

/// File format:
/// size | struct{key, value}
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

        // println!("path {}", path.clone().display());

        let mut filepaths = get_file_paths_sorted(path.clone());

        if filepaths.len() == 0 {
            let fname = format!("0_kv_0.dat");

            let file_path = path.join(fname.clone());

            let _ = new_file(file_path.clone()).unwrap();

            filepaths.push(file_path.clone());
        }

        // println!("filepaths {:?}", &filepaths);

        let readers = get_readers(&filepaths);
        let writer = get_current_writer(&filepaths);
        let curr_gen = get_curr_gen(&filepaths);

        let mut kvstore = KvStore {
            index: Arc::new(Mutex::new(HashMap::new())),
            writer: Arc::new(Mutex::new(StoreWriter {
                dir: path.clone(),
                writer: writer,
                max_segment_size: INITIAL_MAX_SEGMENT_SIZE
                    * u64::pow(NUM_SEGMENTS_COMPACTION_THREASHOLD as u64, curr_gen),
                curr_gen,
            })),
            reader: StoreReader {
                dir: path.clone(),
                readers,
            },
            dir: path.clone(),
            rwmutex: Arc::new(std::sync::RwLock::new(())),
        };

        // println!("{:?}", filepaths);

        kvstore.build_index()?;

        return Ok(kvstore);
    }

    fn build_index(&mut self) -> Result<()> {
        self.index = Arc::new(Mutex::new(HashMap::new()));
        let mut readers = self.reader.clone().readers;
        let mut index = self.index.lock().unwrap();

        for (i, reader) in readers.iter_mut().enumerate() {
            let mut pos = 0;
            let lastpos = reader.seek(SeekFrom::End(0))?;

            while pos < lastpos {
                reader.seek(SeekFrom::Start(pos))?;

                let sz = reader.read_u64::<BigEndian>().unwrap();

                reader.seek(SeekFrom::Start(pos + 8))?;
                let mut buf = vec![0; sz as usize];
                reader.read_exact(&mut buf)?;
                let kvpair: KVPair = serde_json::from_slice(&buf)?;

                index.insert(
                    kvpair.key.clone(),
                    SizeInfo {
                        start: pos + 8,
                        segment_id: i as u32,
                        size: sz,
                    },
                );

                if kvpair.value == "rm".to_string() {
                    if let Some(_) = index.remove(&kvpair.key) {
                    } else {
                        return Err(failure::err_msg("Couldn't delete key"));
                    }
                }

                pos += 8 + sz;
            }
        }

        Ok(())
    }

    // Size-tiered compaction strategy
    fn compact(&self) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        let mut reader = self.reader.clone();

        let compaction_gen = writer.curr_gen + 1;
        let filepaths = get_file_paths_sorted(self.dir.clone());
        let compacted_file_path = self.dir.join(format!("{}_kv_0.dat", compaction_gen));
        let _ = new_file(compacted_file_path.clone())?;
        let mut compaction_writer = get_writer(compacted_file_path.clone());
        let mut index = self.index.lock().unwrap();

        for (key, info) in index.iter_mut() {
            let reader = reader.readers.get_mut(info.segment_id as usize).unwrap();

            reader.seek(SeekFrom::Start(info.start))?;

            let cmd_reader = reader.take(info.size);
            info.start = compaction_writer.seek(SeekFrom::End(0))? + 8;

            compaction_writer.write_u64::<BigEndian>(info.size)?;

            let kvpair: KVPair = serde_json::from_reader(cmd_reader).unwrap();

            write!(compaction_writer, "{}", serde_json::to_string(&kvpair)?)?;

            info.segment_id = 0;
        }
        compaction_writer.flush()?;

        // remove old files
        for filepath in filepaths {
            let _ = fs::remove_file(filepath);
        }

        let mut reader = self.reader.clone();
        reader.readers = get_readers_dir(self.dir.clone());

        writer.writer = compaction_writer;

        let new_rdr = BufReader::new(File::open(compacted_file_path).unwrap());
        reader.readers.push(new_rdr);

        writer.curr_gen = compaction_gen;
        writer.max_segment_size *= NUM_SEGMENTS_COMPACTION_THREASHOLD as u64;

        Ok(())
    }
}

impl KvsEngine for KvStore {
    fn get(&self, key: String) -> Result<Option<String>> {
        let x = self.rwmutex.read().unwrap();
        let index = self.index.lock().unwrap();

        if let Some(info) = index.get(&key) {
            let idx = info.segment_id;
            let mut reader = self.reader.clone();
            let reader = reader.readers.get_mut(idx as usize).unwrap();
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

    fn set(&self, key: String, value: String) -> Result<()> {
        let x = self.rwmutex.write().unwrap();
        let mut writer = self.writer.lock().unwrap();
        let mut reader = self.reader.clone();

        let pos = writer.writer.seek(SeekFrom::End(0))?;
        let num_segments = reader.readers.len();

        if pos >= writer.max_segment_size {
            let fname = format!("{}_kv_{}.dat", writer.curr_gen, num_segments);
            let file_path = self.dir.join(fname);
            new_file(file_path.clone()).unwrap();

            reader.readers.push(get_reader(&file_path));

            writer.writer = get_writer(file_path);
        }

        let pos = writer.writer.seek(SeekFrom::End(0))?;

        let kv_pair = KVPair {
            key: key.clone(),
            value: value.clone(),
        };
        let kv_pair_serialized = serde_json::to_string(&kv_pair)?;

        writer
            .writer
            .write_u64::<BigEndian>(kv_pair_serialized.len() as u64)?;
        write!(writer.writer, "{}", kv_pair_serialized)?;

        writer.writer.flush()?;
        writer.writer.get_ref().sync_all()?;

        let mut index = self.index.lock().unwrap();

        let num_segments = reader.readers.len();
        // println!("num_segments: {}", num_segments);

        index.insert(
            key.clone(),
            SizeInfo {
                start: pos + 8,
                segment_id: (num_segments - 1) as u32,
                size: kv_pair_serialized.len() as u64,
            },
        );

        if reader.readers.len() > NUM_SEGMENTS_COMPACTION_THREASHOLD as usize {
            drop(writer);
            drop(index);

            self.compact()?;
        }

        Ok(())
    }

    fn remove(&self, key: String) -> Result<()> {
        let mut index = self.index.lock().unwrap();

        if let Some(_) = index.remove(&key) {
            drop(index);
            self.set(key, "rm".to_string())
        } else {
            Err(failure::err_msg("Key not found"))
        }
    }
}

fn new_file(path: PathBuf) -> Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .read(true)
        .open(&path)?;

    Ok(file)
}

fn get_readers_dir(path: PathBuf) -> Vec<BufReader<File>> {
    let filepaths = get_file_paths_sorted(path);
    // println!("filepaths: {:?}", filepaths);

    get_readers(&filepaths)
}

fn get_readers(filepaths: &Vec<PathBuf>) -> Vec<BufReader<File>> {
    let mut readers = Vec::new();

    for filepath in filepaths {
        readers.push(get_reader(filepath));
    }

    readers
}

fn get_reader(path: &PathBuf) -> BufReader<File> {
    let file = File::open(path).unwrap();
    BufReader::new(file)
}

fn get_current_writer(filepaths: &Vec<PathBuf>) -> BufWriter<File> {
    let len = filepaths.len();
    let file_path = filepaths[len - 1].clone();

    get_writer(file_path)
}

fn get_writer(path: PathBuf) -> BufWriter<File> {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&path)
        .unwrap();

    BufWriter::new(file)
}

fn get_file_paths_sorted(path: PathBuf) -> Vec<PathBuf> {
    let mut filepaths = Vec::new();

    let mut dir_iter = fs::read_dir(path).unwrap();

    while let Some(entry) = dir_iter.next() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            filepaths.push(path);
        }
    }

    filepaths.sort_by(|a, b| {
        // get filename without extension
        let a_filename = a.file_stem().unwrap().to_str().unwrap();
        let b_filename = b.file_stem().unwrap().to_str().unwrap();

        // get segment id
        let a_filename_split: Vec<&str> = a_filename.split("_").collect();
        let a_segment_id = a_filename_split[2].parse::<u32>().unwrap();

        let b_filename_split: Vec<&str> = b_filename.split("_").collect();
        let b_segment_id = b_filename_split[2].parse::<u32>().unwrap();

        // sort by segment id
        a_segment_id.cmp(&b_segment_id)
    });

    filepaths
}

fn get_curr_gen(filepaths: &Vec<PathBuf>) -> u32 {
    let len = filepaths.len();
    let filepath = filepaths[len - 1].clone();
    let filename = filepath.file_name().unwrap().to_str().unwrap();
    let filename_split: Vec<&str> = filename.split("_").collect();
    filename_split[0].parse::<u32>().unwrap()
}
