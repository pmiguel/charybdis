use std::fs;
use std::fs::File;

struct TableMeta {
    pub file_number: u64,
    pub level: u32
}

struct Manifest {
    curr_file: Option<File>,
    next_sst_id: u64,
    active_sst: Vec<TableMeta>
}

impl Manifest {

    pub fn new() -> Manifest {
        Manifest {
            curr_file: None,
            next_sst_id: 0,
            active_sst: vec![]
        }
    }

    pub fn open(&mut self, path: &str) -> Result<(), std::io::Error>{
        let mut open_options = fs::OpenOptions::new();
        open_options.write(true)
            .append(true)
            .create(true);
        self.curr_file = Some(open_options.open(path).expect("Could not open manifest file."));

        Ok(())
    }
}