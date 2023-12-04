// CSV file reader and modifier using
// Struct, Traits, Trait Bounds, Option, Error, Result, Associate Types
// Ownership and Borrowing to demonstrate Rust understanding,

// Useful Commands:

// Display csv Content
// cargo run -- --read-path=./testdata.csv display

// Paginate display
// cargo run -- --read-path=./testdata.csv paginate 1 3

// delete row of csv
// cargo run -- --read-path=./testdata.csv --write-path=./write.csv delete 1 && cat write.csv

// modify a cell of csv
// cargo run -- --read-path=./testdata.csv --write-path=./write.csv modify -r 1 -c 1 -d yolo && cat write.csv

// modify whole row of csv
// cargo run -- --read-path=./testdata.csv --write-path=./write.csv modify -r 1 -d yolo,this,is,replacement,values,thanks && cat write.csv

use anyhow::{bail, Result};
use clap::Parser;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};
use thiserror::Error;

#[derive(Parser, Debug)]
struct Args {
    // Csv file read path
    #[arg(short, long)]
    read_path: PathBuf,
    // Output data to new csv file or update existing one
    #[arg(short, long)]
    write_path: Option<PathBuf>,
    // Sub command for handling data in csv file
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    // Display entire file
    Display,
    // Paginate(display from row xa to xb)
    Paginate {
        start: usize,
        end: usize,
    },
    // Delete a row/field
    Delete {
        row_index: usize,
    },
    // Modify a row/field
    Modify {
        #[clap(short, long)]
        row_index: usize,

        #[clap(short, long)]
        col_index: Option<usize>,

        // comma seperated values from cli
        #[clap(short, long, value_delimiter = ',')]
        data: Vec<String>,
    },
}

// Trait for data manipulation in CSV.
trait CSVManipulation {
    fn display(&self);
    fn paginate(&self, start: usize, end: usize);
    fn modify(&mut self, row: usize, col: Option<usize>, value: Vec<String>) -> Result<()>;
    fn delete(&mut self, row: usize) -> Result<()>;
}

//Custom errors
#[derive(Error, Debug)]
enum Error {
    #[error("Row index out of bound")]
    RowIndexOutOfBound,
    #[error("Column index out of bound")]
    ColumnIndexOutOfBound,
    #[error("Provided values length mismatch")]
    ValueLengthMismatch,
    #[error("Replacement values length mismatch")]
    ReplacementLengthMismatch,
}

#[allow(dead_code)]
#[derive(Debug)]
struct CSVData {
    data: Vec<Vec<String>>,
    rows: usize,
    cols: usize,
}

impl CSVData {
    pub fn from_file(file_path: PathBuf) -> Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let mut data = Vec::new();
        let mut rows = 0;
        let mut cols = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let row: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if index == 0 {
                cols = row.len();
            }
            data.push(row);
            rows += 1
        }

        Ok(Self { data, rows, cols })
    }

    fn to_file(&self, file_path: PathBuf) -> Result<()> {
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);

        for row in &self.data {
            let line = row.join(",");
            writeln!(writer, "{}", line)?;
        }
        Ok(())
    }

    fn calculate_max_col_width(&self) -> Vec<usize> {
        let mut columns_width = vec![0; self.cols];
        for row in &self.data {
            for (i, cell) in row.iter().enumerate() {
                columns_width[i] = columns_width[i].max(cell.len());
            }
        }
        columns_width
    }

    fn format_row(&self, row: &[String], columns_width: &[usize]) -> String {
        row.iter()
            .enumerate()
            .map(|(index, cell)| {
                let space_padding = columns_width[index] - cell.len();
                format!("{}{}", cell, " ".repeat(space_padding))
            })
            .collect::<Vec<_>>()
            .join("| ")
    }
}

impl CSVManipulation for CSVData {
    fn display(&self) {
        let columns_width = self.calculate_max_col_width();
        for row in &self.data {
            println!("{}", self.format_row(row, &columns_width))
        }
    }

    fn paginate(&self, start: usize, end: usize) {
        let columns_width = self.calculate_max_col_width();
        let buffer_end = end + 1;
        for row in self.data.iter().skip(start - 1).take(buffer_end - start) {
            println!("{}", self.format_row(row, &columns_width))
        }
    }

    fn modify(
        &mut self,
        row_index: usize,
        col_index: Option<usize>,
        values: Vec<String>,
    ) -> Result<()> {
        if row_index == 0 || row_index > self.data.len() {
            bail!(Error::RowIndexOutOfBound);
        }

        match (col_index, values.len()) {
            (Some(index), 1) => {
                if index == 0 || index > self.data[row_index - 1].len() {
                    bail!(Error::ColumnIndexOutOfBound);
                }
                self.data[row_index - 1][index - 1] = format!("\"{}\"", values[0]);
            }
            (None, new_values) => {
                if new_values == self.data[row_index - 1].len() {
                    self.data[row_index - 1] =
                        values.iter().map(|d| format!("\"{}\"", d)).collect();
                } else {
                    bail!(Error::ReplacementLengthMismatch);
                }
            }
            _ => {
                bail!(Error::ValueLengthMismatch)
            }
        }
        Ok(())
    }

    fn delete(&mut self, row_index: usize) -> Result<()> {
        if row_index == 0 || row_index > self.data.len() {
            bail!(Error::RowIndexOutOfBound)
        }
        self.data.remove(row_index - 1);
        Ok(())
    }
}

//Example usage of trait bounds
fn display_data<T: CSVManipulation>(data: &T) {
    data.display();
}

fn paginate_data<T: CSVManipulation>(data: &T, start: usize, end: usize) {
    data.paginate(start, end);
}

fn delete_row<T: CSVManipulation>(data: &mut T, row_index: usize) -> Result<()> {
    data.delete(row_index)?;
    Ok(())
}

fn modify_row<T: CSVManipulation>(
    data: &mut T,
    row_index: usize,
    col_index: Option<usize>,
    values: Vec<String>,
) -> Result<()> {
    data.modify(row_index, col_index, values)?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    let mut csv_data = CSVData::from_file(args.read_path).unwrap();
    match args.command {
        Command::Display => display_data(&csv_data),
        Command::Paginate { start, end } => paginate_data(&csv_data, start, end),
        Command::Delete { row_index } => {
            if let Err(e) = delete_row(&mut csv_data, row_index) {
                panic!("Error occured while deleting row: {}", e)
            }
        }
        Command::Modify {
            row_index,
            col_index,
            data,
        } => {
            if let Err(e) = modify_row(&mut csv_data, row_index, col_index, data) {
                panic!("Error occured while modifying row: {}", e)
            }
        }
    }

    let Some(path) = args.write_path else {
        return;
    };
    if let Err(e) = csv_data.to_file(path) {
        println!("Error occured while wriing to file {}", e);
    };
}
