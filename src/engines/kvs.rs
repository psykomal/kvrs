extern crate failure;
extern crate serde_json;

use crate::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::{collections::HashMap, fs::File, path::PathBuf};

const INITIAL_MAX_SEGMENT_SIZE: u64 = 1024;
const NUM_SEGMENTS_COMPACTION_THREASHOLD: u32 = 4;

pub struct KvStore {
    index: HashMap<String, SizeInfo>,
    writer: BufWriter<File>,
    readers: Vec<BufReader<File>>,
    dir: PathBuf,
    rwmutex: std::sync::RwLock<()>,
    max_segment_size: u64,
    curr_gen: u32,
    num_segments: u32,
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

        let mut filepaths = get_file_paths(path.clone());

        if filepaths.len() == 0 {
            let fname = format!("{}_kv_0.dat", 0);

            let file_path = path.join(fname.clone());

            let _ = new_file(file_path.clone()).unwrap();

            filepaths.push(file_path.clone());
        }

        let readers = get_readers(&filepaths);
        let writer = get_current_writer(&filepaths);
        let num_segments = filepaths.len() as u32;
        let curr_gen = get_curr_gen(&filepaths);

        let mut kvstore = KvStore {
            index: HashMap::new(),
            writer,
            readers,
            dir: path,
            rwmutex: std::sync::RwLock::new(()),
            max_segment_size: INITIAL_MAX_SEGMENT_SIZE
                * u64::pow(NUM_SEGMENTS_COMPACTION_THREASHOLD as u64, curr_gen),
            num_segments,
            curr_gen,
        };

        // println!("{:?}", filepaths);

        kvstore.build_index()?;

        return Ok(kvstore);
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(info) = self.index.get(&key) {
            let idx = info.segment_id;
            let reader = self.readers.get_mut(idx as usize).unwrap();
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

        if pos >= self.max_segment_size {
            let fname = format!("{}_kv_{}.dat", self.curr_gen, self.num_segments);
            let file_path = self.dir.join(fname);
            new_file(file_path.clone()).unwrap();
            self.readers.push(get_reader(&file_path));
            self.num_segments += 1;
            self.writer = get_writer(file_path);
        }

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
        self.writer.get_ref().sync_all()?;
        self.index.insert(
            key.clone(),
            SizeInfo {
                start: pos + 8,
                segment_id: (self.num_segments - 1) as u32,
                size: kv_pair_serialized.len() as u64,
            },
        );

        if self.num_segments > NUM_SEGMENTS_COMPACTION_THREASHOLD {
            self.compact()?;
        }

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
                        segment_id: i as u32,
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

        Ok(())
    }

    // Size-tiered compaction strategy
    fn compact(&mut self) -> Result<()> {
        let x = self.rwmutex.write().unwrap();

        let compaction_gen = self.curr_gen + 1;
        let filepaths = get_file_paths(self.dir.clone());
        let compacted_file_path = self.dir.join(format!("{}_kv_0.dat", compaction_gen));
        let _ = new_file(compacted_file_path.clone())?;
        let mut compaction_writer = get_writer(compacted_file_path.clone());

        for (key, info) in self.index.iter_mut() {
            let reader = self.readers.get_mut(info.segment_id as usize).unwrap();
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
        self.readers = Vec::<BufReader<File>>::new();

        self.writer = compaction_writer;
        let reader = BufReader::new(File::open(compacted_file_path).unwrap());
        self.readers.push(reader);
        self.curr_gen = compaction_gen;
        self.num_segments = 1;
        self.max_segment_size *= NUM_SEGMENTS_COMPACTION_THREASHOLD as u64;

        Ok(())
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

fn get_file_paths(path: PathBuf) -> Vec<PathBuf> {
    let mut filepaths = Vec::new();

    let mut dir_iter = fs::read_dir(path).unwrap();

    while let Some(entry) = dir_iter.next() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            filepaths.push(path);
        }
    }

    filepaths.sort();

    filepaths
}

fn get_curr_gen(filepaths: &Vec<PathBuf>) -> u32 {
    let len = filepaths.len();
    let filepath = filepaths[len - 1].clone();
    let filename = filepath.file_name().unwrap().to_str().unwrap();
    let filename_split: Vec<&str> = filename.split("_").collect();
    filename_split[0].parse::<u32>().unwrap()
}
