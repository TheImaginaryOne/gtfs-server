#![allow(unused)]
#![allow(clippy::all)]

use crate::schema::*;
use chrono::prelude::*;
use diesel::deserialize::QueryableByName;
use diesel::pg::types::sql_types::Timestamptz;
use diesel::sql_types::{Bool, Integer, Nullable, Text};
use diesel::{Identifiable, Queryable};
use serde::Serialize;

#[derive(QueryableByName, Debug, Serialize)]
pub struct StopTimeByStop {
    #[sql_type = "Text"]
    stop_id: String,
    #[sql_type = "Text"]
    trip_id: String,
    #[sql_type = "Timestamptz"]
    departure_time: DateTime<Utc>,
    #[sql_type = "diesel::sql_types::Date"]
    service_date: NaiveDate,
    #[sql_type = "Integer"]
    stop_sequence: i32,
    #[sql_type = "Nullable<Bool>"]
    direction_id: Option<bool>,
    #[sql_type = "Nullable<Text>"]
    trip_headsign: Option<String>,
    #[sql_type = "Nullable<Text>"]
    route_short_name: Option<String>,
    #[sql_type = "Text"]
    route_long_name: String,
    #[sql_type = "Integer"]
    route_type: i32,
}
