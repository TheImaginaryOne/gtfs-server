use dotenv::dotenv;
use postgres::{Client, NoTls};

use std::error::Error;
use std::io::Write;
use std::path::Path;

use structopt::StructOpt;

use derive_more::{Display, From};

static TABLE_AND_FILE_NAMES: [(&str, &str); 8] = [
    ("shapes.txt", "shape"),
    ("agency.txt", "agency"),
    ("routes.txt", "route"),
    ("trips.txt", "trip"),
    ("calendar.txt", "calendar"),
    ("calendar_dates.txt", "calendar_date"),
    ("stops.txt", "stop"),
    ("stop_times.txt", "stop_time"),
];

#[derive(Debug, From, Display)]
enum ImporterError {
    #[display(fmt = "Database error: {}", _0)]
    DbError(postgres::Error),
    #[display(fmt = "File error: {}", _0)]
    FileError(std::io::Error),
    #[display(fmt = "{} file should have data", _0)]
    NoDataInFile(String),
    #[display(fmt = "Error while parsing csv: {}", _0)]
    CsvError(csv::Error),
}
impl Error for ImporterError {}

#[derive(StructOpt, Debug)]
#[structopt(name = "gtfs_postgres_importer")]
enum Options {
    Import {
        #[structopt(short, long)]
        path: String,
    },
    DeleteFeed {
        #[structopt(short="f", long)]
        feed_id: u32,
    }
}
fn main() {
    match run() {
        Ok(()) => println!("Successful!"),
        Err(e) => eprintln!("{}", e),
    }
}
fn run() -> Result<(), ImporterError> {
    let options = Options::from_args();

    dotenv().ok();

    let db_url = &std::env::var("DATABASE_URL").expect("env var DATABASE_URL expected to be set");

    println!("Connecting to {}", db_url);
    let mut client = Client::connect(db_url, NoTls)?;

    match options {
        Options::Import {path} => import(path, &mut client),
        Options::DeleteFeed {feed_id} => delete_feed(feed_id, &mut client)
    }
}

fn delete_feed(feed_id: u32, client: &mut Client) -> Result<(), ImporterError> {
    let mut transaction = client.transaction()?;
    println!("Deleting data with feed_id = {}", feed_id);

    // rev to avoid foreign key violations
    for s in TABLE_AND_FILE_NAMES.iter().rev() {
        println!("Deleting from table {}", &s.1);
        transaction.execute(
            &format!("delete from {} where feed_id={}", &s.1, &feed_id)[..],
            &[],
        )?;
    }
    println!("Deleting from table feed");
    transaction.execute(
        &format!("delete from feed where feed_id={}", &feed_id)[..],
        &[],
    )?;

    transaction.commit()?;
    Ok(())
}

fn import(p: String, client: &mut Client) -> Result<(), ImporterError> {
    let path = Path::new(&p);

    println!("Importing data");

    let mut transaction = client.transaction()?;

    let feed_id: i32 = transaction
        .query("insert into feed default values returning feed_id", &[])?
        .first()
        .unwrap()
        .get(0);

    for s in &TABLE_AND_FILE_NAMES {
        transaction.execute(
            &format!(
                "alter table {} alter column feed_id set default {}",
                s.1, feed_id
            )[..],
            &[],
        )?;

        let file_path = path.join(&s.0);
        println!("Reading from {}", &file_path.display());

        let file_content = std::fs::read_to_string(&file_path)?;

        // header is the csv header, we put this in the copy command
        // so that the csv data is input correctly.
        let mut header = match file_content.find(|c| c == '\n') {
            Some(i) => &file_content[..i],
            None => return Err(ImporterError::NoDataInFile(file_path.display().to_string())),
        };

        if header.ends_with('\r') {
            header = &header[0..header.len() - 1];
        }
        // strip BOM (ef bb bf) if present
        if header.bytes().nth(0) == Some(239u8)
            && header.bytes().nth(1) == Some(187u8)
            && header.bytes().nth(2) == Some(191u8)
        {
            header = &header[3..header.len()];
        }

        let command = format!(
            "copy {}({}) from stdin delimiter ',' csv header;",
            s.1, header
        );
        println!("Running: {}", &command);

        let mut writer = transaction.copy_in(&command[..])?;

        println!("Writing data");

        if s.1 == "stop_time" {
            let mut reader = csv::Reader::from_reader(file_content.as_bytes());
            let mut csv_writer = csv::Writer::from_writer(&mut writer);
            let header_columns = header.split(',').collect::<Vec<_>>();
            let arrival_time_idx = header_columns
                .iter()
                .position(|r| *r == "arrival_time")
                .unwrap();
            let departure_time_idx = header_columns
                .iter()
                .position(|r| *r == "departure_time")
                .unwrap();

            for row in reader.records() {
                let record = row.unwrap(); // todo, ugly code!
                let new_record = record
                    .iter()
                    .enumerate()
                    .map(|(i, content)| {
                        if i == arrival_time_idx || i == departure_time_idx {
                            let x = content
                                .split(':')
                                .map(|x| x.parse::<i32>().unwrap())
                                .collect::<Vec<i32>>();
                            (x[0] * 3600 + x[1] * 60 + x[2]).to_string()
                        } else {
                            content.to_string()
                        }
                    })
                    .collect::<Vec<String>>();
                csv_writer.write_record(new_record)?;
            }
        } else {
            writer.write_all(file_content.as_bytes())?;
        }
        println!("Committing to database");

        writer.finish()?;

        transaction.execute(
            &format!("alter table {} alter column feed_id drop default", s.1)[..],
            &[],
        )?;
    }
    transaction.commit()?;
    Ok(())
}
