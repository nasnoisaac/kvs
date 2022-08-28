use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::error::{KvsError, Result};

/// The KvStore is a simple key-value store.
/// # Example:
/// ```rust
/// use kvs::KvStore;
///
/// let mut store = KvStore::new();
///
/// store.set("key".to_string(), "test".to_string());
/// store.get("key".to_string());
/// store.remove("key".to_string());
/// ```
pub struct KvStore {
    path: PathBuf,
    index: BTreeMap<String, CommandPos>, // store command pos
    reader: BufReader<File>,
    writer: BufWriterWithPos<File>,
}

#[derive(Debug)]
pub struct CommandPos {
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

        // get log file
        let log_file = log_file(&path);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&log_file)?;

        let mut reader = BufReader::new(file.try_clone()?);
        load(&mut reader, &mut index);
        let writer = BufWriterWithPos::new(file)?;

        Ok(KvStore {
            path,
            index,
            writer,
            reader,
        })
    }

    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        let command = Command::Set {
            key: key.clone(),
            val: val.clone(),
        };
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;
        let cmd_pos = CommandPos {
            pos,
            len: self.writer.pos - pos,
        };
        debug!("Set cmd: {:?}", cmd_pos);
        self.index.insert(key, cmd_pos);
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd) = self.index.get(&key) {
            // self.reader.seek(SeekFrom::Start(*pos))?;
            let reader = self.reader.get_mut();
            reader.seek(SeekFrom::Start(cmd.pos))?;
            let cmd_reader = reader.take(cmd.len);
            debug!("Read cmd: {:?}", cmd);
            if let Command::Set { val, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(val))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if let Some(res) = self.index.remove(&key) {
            let command = Command::Rm { key: key.clone() };
            let pos = self.writer.pos;
            serde_json::to_writer(&mut self.writer, &command)?;
            self.writer.flush()?;
            Ok(())
        } else {
            Err(KvsError::KeyNotFound(key))
        }
    }
}

fn load(reader: &mut BufReader<File>, index: &mut BTreeMap<String, CommandPos>) -> Result<()> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    info!("Load File");
    while let Some(command) = stream.next() {
        debug!("{:?}", command);
        let new_pos = stream.byte_offset() as u64;
        match command {
            Ok(Command::Set { key, val }) => {
                index.insert(
                    key,
                    CommandPos {
                        pos,
                        len: new_pos - pos,
                    },
                );
            }
            Ok(Command::Rm { key }) => {
                index.remove(&key);
            }
            Err(_) => unreachable!(),
        }
        pos = new_pos;
    }
    Ok(())
}

fn log_file(path: &PathBuf) -> PathBuf {
    path.join("temp.log")
}


struct BufWriterWithPos<T: Write + Seek> {
    writer: BufWriter<T>,
    pos: u64,
}

impl<T: Write + Seek> BufWriterWithPos<T> {
    fn new(mut inner: T) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos{
            writer: BufWriter::new(inner),
            pos
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
