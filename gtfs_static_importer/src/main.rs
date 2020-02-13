use bytes::buf::BufMutExt;
use bytes::BytesMut;
use dotenv::dotenv;
use postgres::{Client, NoTls};
use tokio::io::AsyncWriteExt;

use std::error::Error;
use std::io::Write;
use std::path::Path;

use structopt::StructOpt;

use derive_more::{Display, From};

mod utils;

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
    #[display(fmt = "Error reading env var {}: {}", _0, _1)]
    EnvVar(String, std::env::VarError),
    #[display(fmt = "Http error: {}", _0)]
    Http(reqwest::Error),
    #[display(fmt = "Zip extraction error: {}", _0)]
    Zip(zip::result::ZipError),
    #[display(fmt = "Tokio error: {}", _0)]
    TokioJoin(tokio::task::JoinError),
}
impl Error for ImporterError {}

#[derive(StructOpt, Debug)]
#[structopt(name = "gtfs_postgres_importer")]
enum Options {
    Import {
        #[structopt(short, long)]
        path: String,
    },
    Download {
        #[structopt(short = "f", long)]
        tf_feed_id: String,
    },
    DeleteFeed {
        #[structopt(short = "f", long)]
        feed_id: u32,
    },
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

    let db_url = &std::env::var("DATABASE_URL")
        .map_err(|e| ImporterError::EnvVar("DATABASE_URL".into(), e))?;

    println!("Connecting to {}", db_url);
    let mut client = postgres::Client::connect(db_url, NoTls)?;

    match options {
        Options::Import { path } => import(&Path::new(&path), &mut client),
        Options::DeleteFeed { feed_id } => delete_feed(feed_id, &mut client),
        Options::Download { tf_feed_id } => download(tf_feed_id, &mut client),
    }
}
// todo async
fn delete_feed(feed_id: u32, client: &mut Client) -> Result<(), ImporterError> {
    let mut transaction = client.transaction()?;
    println!("Deleting data with feed_id = {}", feed_id);

    let bar = utils::progress_bar(TABLE_AND_FILE_NAMES.len() as u64, "Deleting {spinner} [{elapsed_precise}] [{bar:60.yellow}] {pos}/{len}");
    // rev to avoid foreign key violations
    for s in TABLE_AND_FILE_NAMES.iter().rev() {
        bar.println(format!("Deleting from table {}", &s.1));
        transaction.execute(
            &format!("delete from {} where feed_id={}", &s.1, &feed_id)[..],
            &[],
        )?;
        bar.inc(1);
    }
    transaction.execute(
        &format!("delete from feed where feed_id={}", &feed_id)[..],
        &[],
    )?;
    transaction.commit()?;
    bar.finish_and_clear();
    
    Ok(())
}

fn download(feed_id: String, client: &mut Client) -> Result<(), ImporterError> {
    let tf_key = &std::env::var("TRANSITFEEDS_KEY")
        .map_err(|e| ImporterError::EnvVar("TRANSITFEEDS_KEY".into(), e))?;

    let temp_file = tempfile::tempfile()?;
    let mut async_file = tokio::fs::File::from_std(temp_file);

    let mut runtime = tokio::runtime::Runtime::new()?;

    println!("Downloading latest feed");

    // TODO: workaround, reqwest has no blocking Response::chunk()
    runtime.block_on(async {
        let client = reqwest::Client::new();

        let mut response = client
            .get("https://api.transitfeeds.com/v1/getLatestFeedVersion")
            .query(&[("key".to_string(), tf_key), ("feed".to_string(), &feed_id)])
            .send()
            .await?;
        dbg!(response.headers());
        if let Some(l) = response.content_length() {
            let bar = utils::progress_bar(l as u64, "Downloading {spinner} [{elapsed_precise}] [{bar:60.yellow}] {bytes}/{total_bytes}");
            while let Some(chunk) = response.chunk().await? {
                bar.inc(chunk.len() as u64);
                async_file.write(&chunk).await?;
            }
        }
        Ok::<(), ImporterError>(())
    })?;

    // unwrap should work, as we have finished all io operations.

    let temp_folder = tempfile::tempdir()?;
    let temp_folder_path = temp_folder.path();

    let mut zip = zip::ZipArchive::new(async_file.try_into_std().unwrap())?;

    let bar = utils::progress_bar(zip.len() as u64, "Writing {spinner} [{elapsed_precise}] [{bar:60.yellow}] {pos}/{len}");

    for i in 0..zip.len() {
        let mut inner = zip.by_index(i)?;

        let file_name = inner.sanitized_name();

        bar.println(format!("Writing {}", file_name.display()));

        let file_path = std::path::PathBuf::from(temp_folder_path).join(file_name);
        let mut new_file = std::fs::File::create(file_path)?;

        std::io::copy(&mut inner, &mut new_file)?;
        bar.inc(1);
    }
    bar.finish_and_clear();
    import(temp_folder_path, client)?;

    Ok(())
}

fn import(path: &Path, client: &mut Client) -> Result<(), ImporterError> {

    println!("Importing data");

    let mut transaction = client.transaction()?;

    let feed_id: i32 = transaction
        .query("insert into feed default values returning feed_id", &[])?
        .first()
        .unwrap()
        .get(0);

    let bar = utils::progress_bar(TABLE_AND_FILE_NAMES.len() as u64, "Importing {spinner} [{elapsed_precise}] [{bar:60.yellow}] {pos}/{len}");

    // TODO support optional tables
    for s in &TABLE_AND_FILE_NAMES {
        transaction.execute(
            &format!(
                "alter table {} alter column feed_id set default {}",
                s.1, feed_id
            )[..],
            &[],
        )?;

        let file_path = path.join(&s.0);
        bar.println(format!("Reading from {}", &file_path.display()));

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
        bar.println(format!("Running: {}", &command));

        let mut writer = transaction.copy_in(&command[..])?;

        bar.println("Writing data");

        if s.1 == "stop_time" {
            let mut buffer = BytesMut::new().writer();
            {
                let mut reader = csv::Reader::from_reader(file_content.as_bytes());
                let mut csv_writer = csv::Writer::from_writer(&mut buffer);
                let header_columns = header.split(',').collect::<Vec<_>>();
                let arrival_time_idx = header_columns.iter().position(|r| *r == "arrival_time");
                let departure_time_idx = header_columns.iter().position(|r| *r == "departure_time");

                for row in reader.records() {
                    let record = row?; // todo, ugly code!
                    let new_record = record
                        .iter()
                        .enumerate()
                        .map(|(i, content)| {
                            if Some(i) == arrival_time_idx || Some(i) == departure_time_idx {
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
            }
            writer.write_all(&buffer.into_inner())?;
        } else {
            writer.write_all(file_content.as_bytes())?;
        }
        bar.println("Committing to database");

        writer.finish()?;

        transaction.execute(
            &format!("alter table {} alter column feed_id drop default", s.1)[..],
            &[],
        )?;
        bar.inc(1);
    }
    bar.finish_and_clear();
    transaction.commit()?;
    Ok(())
}
