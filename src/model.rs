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
    pub stop_id: String,
    #[sql_type = "Text"]
    pub trip_id: String,
    #[sql_type = "Timestamptz"]
    pub departure_time: DateTime<Utc>,
    #[sql_type = "diesel::sql_types::Date"]
    pub service_date: NaiveDate,
    #[sql_type = "Integer"]
    pub stop_sequence: i32,
    #[sql_type = "Nullable<Bool>"]
    pub direction_id: Option<bool>,
    #[sql_type = "Nullable<Text>"]
    pub trip_headsign: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub route_short_name: Option<String>,
    #[sql_type = "Text"]
    pub route_long_name: String,
    #[sql_type = "Integer"]
    pub route_type: i32,
}
