use clap::Parser;
use color_eyre::eyre::{WrapErr, Result};
use std::cmp::max;
use std::{fs::File, io};


/// Pretty print the CSV, still in csv format
#[derive(Parser, Debug)]
pub struct FixwidthArgs {
    input_file: String,

    /// File to output to, if not specified standard out will be used
    #[arg(short, long)]
    out: Option<String>,
}

pub fn csvfixwidth(args: FixwidthArgs) -> Result<()> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(&args.input_file)?;
    let mut col_sizes = vec![];
    for file_row in reader.records() {
        let row = &file_row.wrap_err(format!("Failed to parse {}", &args.input_file))?;
        let additional = (row.len() as i32) - (col_sizes.len() as i32);
        if let Ok(pos_additional) = additional.try_into() {
            col_sizes.extend(vec![0; pos_additional]);
        }
        for (i, val) in row.iter().enumerate() {
            col_sizes[i] = max(col_sizes[i], val.trim().len())
        }
    }

    let mut writer: csv::Writer<Box<dyn io::Write>> = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(match args.out {
            Some(path) => Box::new(File::create(path)?),
            None => Box::new(io::stdout()),
        });

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(&args.input_file)?;
    for record in reader.records() {
        let row = &record?;
        let padding: Vec<_> = row
            .iter()
            .zip(&col_sizes)
            .map(|(cell, size)| {
                let trimmed =  cell.trim();
                if trimmed.is_empty() {
                    0
                } else {
                    size - trimmed.len()
                }
            })
            .collect();
        let rec = row.iter().enumerate().map(|(i, cell)| {
            if i == 0 {
                cell.trim().to_owned()
            } else if padding[i] > 0 {
                format!("{:w$}{}", "", cell.trim(), w = padding[i - 1] + 1)
            } else {
                format!(
                    "{:w1$}{:w2$}",
                    "",
                    cell.trim(),
                    w1 = padding[i - 1] + 1,
                    w2 = col_sizes[i]
                )
            }
        });
        writer.write_record(rec)?;
    }

    Ok(())
}
