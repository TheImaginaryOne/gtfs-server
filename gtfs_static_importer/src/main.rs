use bytes::buf::BufMutExt;
use bytes::{Bytes, BytesMut};
use dotenv::dotenv;
use futures_util::sink::SinkExt;
use tokio::io::AsyncWriteExt;
use tokio_postgres::{Client, NoTls};

use std::error::Error;
use std::path::Path;

use structopt::StructOpt;

use derive_more::{Display, From};

use indicatif::{ProgressBar, ProgressStyle};

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
    DbError(tokio_postgres::Error),
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
#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => println!("Successful!"),
        Err(e) => eprintln!("{}", e),
    }
}
async fn run() -> Result<(), ImporterError> {
    let options = Options::from_args();

    dotenv().ok();

    let db_url = &std::env::var("DATABASE_URL")
        .map_err(|e| ImporterError::EnvVar("DATABASE_URL".into(), e))?;

    println!("Connecting to {}", db_url);
    let (mut client, connection) = tokio_postgres::connect(db_url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    match options {
        Options::Import { path } => import(path, &mut client).await,
        Options::DeleteFeed { feed_id } => delete_feed(feed_id, &mut client).await,
        Options::Download { tf_feed_id } => download(tf_feed_id, &mut client).await,
    }
}
// todo async
async fn delete_feed(feed_id: u32, client: &mut Client) -> Result<(), ImporterError> {
    let transaction = client.transaction().await?;
    println!("Deleting data with feed_id = {}", feed_id);

    // rev to avoid foreign key violations
    for s in TABLE_AND_FILE_NAMES.iter().rev() {
        println!("Deleting from table {}", &s.1);
        transaction
            .execute(
                &format!("delete from {} where feed_id={}", &s.1, &feed_id)[..],
                &[],
            )
            .await?;
    }
    println!("Deleting from table feed");
    transaction
        .execute(
            &format!("delete from feed where feed_id={}", &feed_id)[..],
            &[],
        )
        .await?;

    transaction.commit().await?;
    Ok(())
}

async fn download(feed_id: String, _client: &mut Client) -> Result<(), ImporterError> {
    let tf_key = &std::env::var("TRANSITFEEDS_KEY")
        .map_err(|e| ImporterError::EnvVar("TRANSITFEEDS_KEY".into(), e))?;

    let mut async_file = tokio::task::spawn_blocking(|| {
        let temp_file = tempfile::tempfile()?;
        Ok::<tokio::fs::File, ImporterError>(tokio::fs::File::from_std(temp_file))
    })
    .await??;

    println!("Downloading latest feed");

    let client = reqwest::Client::new();

    let mut response = client
        .get("https://api.transitfeeds.com/v1/getLatestFeedVersion")
        .query(&[("key".to_string(), tf_key), ("feed".to_string(), &feed_id)])
        .send()
        .await?;

    if let Some(l) = response.content_length() {
        let bar = ProgressBar::new(l);
        bar.set_style(ProgressStyle::default_bar().progress_chars("=> ").template(
            "Downloading {spinner} [{elapsed_precise}] [{bar:60.yellow}] {bytes}/{total_bytes}",
        ));

        while let Some(chunk) = response.chunk().await? {
            bar.inc(chunk.len() as u64);
            async_file.write(&chunk).await?;
        }
    }

    // unwrap should work, as we have finished all async io operations.
    let mut zip = zip::ZipArchive::new(async_file.try_into_std().unwrap())?;

    let temp_folder = tempfile::tempdir()?;
    let temp_folder_path = temp_folder.path();

    let bar = ProgressBar::new(zip.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .progress_chars("=> ")
            .template("Writing {spinner} [{elapsed_precise}] [{bar:60.yellow}] {pos}/{len}"),
    );

    for i in 0..zip.len() {
        let mut inner = zip.by_index(i)?;

        let file_name = inner.sanitized_name();

        bar.println(format!("Writing {}", file_name.display()));

        let file_path = std::path::PathBuf::from(temp_folder_path).join(file_name);
        let mut new_file = std::fs::File::create(file_path)?;
        std::io::copy(&mut inner, &mut new_file)?;

        bar.inc(1);
    }

    Ok(())
}

async fn import(p: String, client: &mut Client) -> Result<(), ImporterError> {
    let path = Path::new(&p);

    println!("Importing data");

    let transaction = client.transaction().await?;

    let feed_id: i32 = transaction
        .query("insert into feed default values returning feed_id", &[])
        .await?
        .first()
        .unwrap()
        .get(0);

    for s in &TABLE_AND_FILE_NAMES {
        transaction
            .execute(
                &format!(
                    "alter table {} alter column feed_id set default {}",
                    s.1, feed_id
                )[..],
                &[],
            )
            .await?;

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

        let writer = transaction.copy_in(&command[..]).await?;
        futures::pin_mut!(writer);

        println!("Writing data");

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
            writer.send(buffer.into_inner().freeze()).await?;
        } else {
            writer.send(Bytes::from(file_content)).await?;
        }
        println!("Committing to database");

        writer.finish().await?;

        transaction
            .execute(
                &format!("alter table {} alter column feed_id drop default", s.1)[..],
                &[],
            )
            .await?;
    }
    transaction.commit().await?;
    Ok(())
}
