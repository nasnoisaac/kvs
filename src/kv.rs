use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::error::{KvsError, Result};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// The KvStore is a simple key-value store.
/// # Example:
/// ```rust
/// # use kvs::{KvStore, Result};
/// # fn try_main() -> Result<()> {
/// use std::env::current_dir;
/// let mut store = KvStore::open(current_dir()?)?;
/// store.set("key".to_owned(), "value".to_owned())?;
/// let val = store.get("key".to_owned())?;
/// assert_eq!(val, Some("value".to_owned()));
/// # Ok(())
/// # }
/// ```
pub struct KvStore {
    path: PathBuf,
    index: BTreeMap<String, CommandPos>, // store command pos
    readers: HashMap<u64, BufReader<File>>,
    writer: BufWriterWithPos<File>,
    current_gen: u64,
    uncompacted: u64,
}

#[derive(Debug)]
pub struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Command {
    Set { key: String, val: String },
    Rm { key: String },
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;

        let mut index = BTreeMap::new();
        let mut readers = HashMap::new();

        let gen_list = sorted_gen_list(&path)?;
        let mut uncompacted = 0;

        for &gen in &gen_list {
            let mut reader = BufReader::new(File::open(log_file(&path, gen))?);
            uncompacted += load(gen, &mut reader, &mut index)?;
            readers.insert(gen, reader);
        }
        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file_writer(&path, current_gen, &mut readers)?;

        Ok(KvStore {
            path,
            index,
            writer,
            current_gen,
            readers,
            uncompacted,
        })
    }

    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        let command = Command::Set {
            key: key.clone(),
            val,
        };
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;
        let cmd_pos = CommandPos {
            gen: self.current_gen,
            pos,
            len: self.writer.pos - pos,
        };
        debug!("Set cmd: {:?}", cmd_pos);
        if let Some(old_cmd) = self.index.insert(key, cmd_pos) {
            self.uncompacted += old_cmd.len;
        }
        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd) = self.index.get(&key) {
            // self.reader.seek(SeekFrom::Start(*pos))?;
            let reader = self.readers.get_mut(&cmd.gen).unwrap();
            reader.seek(SeekFrom::Start(cmd.pos))?;
            let cmd_reader = reader.take(cmd.len);
            debug!("Read cmd: {:?}", cmd);
            if let Command::Set { val, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(val))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if let Some(_res) = self.index.remove(&key) {
            let command = Command::Rm { key: key.clone() };
            serde_json::to_writer(&mut self.writer, &command)?;
            self.writer.flush()?;
            Ok(())
        } else {
            Err(KvsError::KeyNotFound(key))
        }
    }

    pub fn compact(&mut self) -> Result<()> {
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 1;
        self.writer = self.new_log_file_writer(self.current_gen)?;

        let mut compaction_writer = self.new_log_file_writer(compaction_gen)?;

        let mut new_pos = 0;
        for cmd_pos in self.index.values_mut() {
            let reader = self.readers.get_mut(&cmd_pos.gen).unwrap();
            reader.seek(SeekFrom::Start(cmd_pos.pos))?;

            let mut entry_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut compaction_writer)?;
            *cmd_pos = CommandPos {
                gen: compaction_gen,
                pos: new_pos,
                len,
            };
            new_pos += len;
        }

        // remove stale log files.
        let stale_gens: Vec<_> = self
            .readers
            .keys()
            .filter(|&&gen| gen < compaction_gen)
            .cloned()
            .collect();
        for stale_gen in stale_gens {
            self.readers.remove(&stale_gen);
            fs::remove_file(log_file(&self.path, stale_gen))?;
        }
        self.uncompacted = 0;

        Ok(())
    }

    fn new_log_file_writer(&mut self, gen: u64) -> Result<BufWriterWithPos<File>> {
        new_log_file_writer(&self.path, gen, &mut self.readers)
    }
}

fn sorted_gen_list(path: &Path) -> Result<Vec<u64>> {
    let mut gen_list = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Ok(gen) = path.file_stem().unwrap().to_str().unwrap().parse::<u64>() {
                gen_list.push(gen);
            }
        }
    }
    gen_list.sort();
    Ok(gen_list)
}

fn new_log_file_writer(
    path: &Path,
    gen: u64,
    readers: &mut HashMap<u64, BufReader<File>>,
) -> Result<BufWriterWithPos<File>> {
    let path = log_file(path, gen);
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;
    readers.insert(gen, BufReader::new(File::open(&path)?));
    Ok(writer)
}

fn load(
    gen: u64,
    reader: &mut BufReader<File>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0;
    info!("Load File");
    while let Some(command) = stream.next() {
        debug!("{:?}", command);
        let new_pos = stream.byte_offset() as u64;
        match command {
            Ok(Command::Set { key, val:_ }) => {
                let cmd_pos = CommandPos {
                    gen,
                    pos,
                    len: new_pos - pos,
                };
                if let Some(old_cmd) = index.insert(key, cmd_pos) {
                    // compact old command
                    uncompacted += old_cmd.len;
                }
            }
            Ok(Command::Rm { key }) => {
                if let Some(old_cmd) = index.remove(&key) {
                    // compact old command
                    uncompacted += old_cmd.len;
                }
                // compact removed value
                uncompacted += new_pos - pos;
            }
            Err(_) => unreachable!(),
        }
        pos = new_pos;
    }
    Ok(uncompacted)
}

fn log_file(path: &Path, gen: u64) -> PathBuf {
    path.join(format!("{}.log", gen))
}

struct BufWriterWithPos<T: Write + Seek> {
    writer: BufWriter<T>,
    pos: u64,
}

impl<T: Write + Seek> BufWriterWithPos<T> {
    fn new(mut inner: T) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<T: Write + Seek> Write for BufWriterWithPos<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.writer.write(buf);
        if let Ok(size) = res {
            self.pos += size as u64;
        }
        res
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
