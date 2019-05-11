use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

use bincode::{deserialize, deserialize_from, serialize_into};

use crate::data::{CommitLog, Data};

pub struct ConfigFile {
    data: Data,
    file: File,
}

impl ConfigFile {
    pub fn new(file: String) -> Self {
        let path = Path::new(&file);
        let open_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path);

        let mut file = match open_file {
            Ok(f) => f,
            Err(_) => {
                File::create(path).expect("Config file not creatable")
            }
        };

        let reader = BufReader::new(&file);

        let data = match deserialize_from(reader) {
            Err(e) => {
                Data::new()
            }
            Ok(e) => e
        };

        println!("Data is {:?}", &data);

        ConfigFile { file, data }
    }

    pub fn save(&mut self) {
        let mut f = BufWriter::new(&self.file);
        serialize_into(&mut f, &self.data).unwrap();
        f.flush().unwrap();
    }

    pub fn add_log(&mut self, log: CommitLog) {
        self.data.add(log);
        self.save();
    }
}
