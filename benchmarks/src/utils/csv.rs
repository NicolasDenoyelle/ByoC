use std::collections::BTreeMap;
use std::io::Write;

pub enum Error {
    MissingField,
    InvalidNumberOfFields,
}

pub type Record = BTreeMap<String, String>;

// pub fn make_profiler_record<C>(profiler: byoc::Profiler<C>) -> Record {
//     let record = Record::new()
// }

pub struct Table {
    header: Vec<String>,
    records: Vec<Vec<String>>,
}

impl Table {
    pub fn new(colnames: Vec<String>) -> Self {
        Self {
            header: colnames,
            records: Vec::new(),
        }
    }

    pub fn push(&mut self, mut record: Record) -> Result<(), Error> {
        if self.header.len() != record.len() {
            return Err(Error::InvalidNumberOfFields);
        }

        let mut table_record = Vec::<String>::with_capacity(record.len());
        for colname in self.header.iter() {
            match record.remove(colname) {
                Some(value) => {
                    table_record.push(value);
                }
                None => return Err(Error::MissingField),
            }
        }
        self.records.push(table_record);

        Ok(())
    }

    pub fn write<W: Write>(
        &self,
        output: &mut W,
        separator: &str,
    ) -> Result<(), std::io::Error> {
        writeln!(output, "{}", self.header.join(separator))?;
        for record in self.records.iter() {
            writeln!(output, "{}", record.clone().join(separator))?;
        }
        Ok(())
    }
}
