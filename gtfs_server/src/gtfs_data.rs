use crate::protobuf::gtfs_realtime::{FeedMessage, TripUpdate, VehicleDescriptor};
use chrono::NaiveDate;
use log::warn;
// used because Equivalent trait is more flexible than Borrow trait.
use indexmap::{Equivalent, IndexMap};
use serde::Serialize;

#[derive(PartialEq, Eq, Hash)]
struct TripUpdateKey(NaiveDate, String);
#[derive(PartialEq, Eq, Hash)]
struct TripUpdateKeyRef<'a>(NaiveDate, &'a str);

impl Equivalent<TripUpdateKey> for TripUpdateKeyRef<'_> {
    fn equivalent(&self, k: &TripUpdateKey) -> bool {
        self.0 == k.0 && self.1 == k.1
    }
}

pub struct RealtimeUpdateManager {
    trip_updates: IndexMap<TripUpdateKey, TripUpdate>,
}

#[allow(dead_code)]
impl RealtimeUpdateManager {
    pub fn new() -> Self {
        Self {
            trip_updates: IndexMap::new(),
        }
    }
    pub fn load_feed(&mut self, feed: FeedMessage) {
        self.trip_updates.clear();
        for entity in feed.entity {
            if let Some(trip_update) = entity.trip_update {
                let trip_id = match trip_update.trip.trip_id.clone() {
                    Some(id) => id,
                    None => {
                        warn!("No trip_id found for a TripDescriptor");
                        continue;
                    }
                };

                let start_date = match &trip_update.trip.start_date {
                    Some(s) => match NaiveDate::parse_from_str(&s, "%Y%m%d") {
                        Ok(d) => d,
                        Err(e) => {
                            warn!("Error parsing date \"{}\": {}", s, e);
                            continue;
                        }
                    },
                    None => {
                        warn!("No start_date found for a TripDescriptor");
                        continue;
                    }
                };

                self.trip_updates
                    .insert(TripUpdateKey(start_date, trip_id.clone()), trip_update);
            }
        }
    }
    pub fn get_realtime_updates<'a, I: IntoIterator<Item = RealtimeQueryKey<'a>>>(
        &self,
        keys: I,
    ) -> Vec<Option<RealtimeUpdate>> {
        keys.into_iter()
            .map(|key| {
                match self
                    .trip_updates
                    .get(&TripUpdateKeyRef(key.start_date, &key.trip_id))
                {
                    Some(trip_update) => {
                        let mut realtime_data = RealtimeUpdate {
                            delay: None,
                            schedule_relationship: None,
                            vehicle: None,
                        };

                        let stop_time_update_opt = trip_update
                            .stop_time_update // Vec<StopTimeUpdate> with all updates for the current trip
                            .iter()
                            .take_while(|s| Some(key.stop_sequence) >= s.stop_sequence)
                            .last();
                        //dbg!(&trip_update.trip, &trip_update.stop_time_update, &stop_time_update_opt, &key.stop_sequence);

                        if let Some(stop_time_update) = stop_time_update_opt {

                            if let Some(stop_event) = &stop_time_update.departure {
                                if let Some(delay) = stop_event.delay {
                                    realtime_data.delay = Some(delay);
                                }
                            } else {
                                if let Some(stop_event) = &stop_time_update.arrival {
                                    if let Some(delay) = stop_event.delay {
                                        realtime_data.delay = Some(delay);
                                    } else {
                                        warn!("No departure delay info found");
                                    }
                                }
                            }
                            realtime_data.schedule_relationship =
                                stop_time_update.schedule_relationship;
                        }
                        realtime_data.vehicle = trip_update.vehicle.clone();
                        Some(realtime_data)
                    }
                    None => None,
                }
            })
            .collect::<Vec<_>>()
    }
}

/// Any better name?
pub struct RealtimeQueryKey<'a> {
    pub start_date: NaiveDate,
    pub trip_id: &'a str,
    pub stop_sequence: u32,
}

#[derive(PartialEq, Debug, Serialize)]
pub struct RealtimeUpdate {
    /// The delay in seconds.
    pub delay: Option<i32>,
    pub schedule_relationship: Option<i32>,
    pub vehicle: Option<VehicleDescriptor>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protobuf::gtfs_realtime::{trip_update::*, *};
    fn h() -> FeedHeader {
        FeedHeader {
            gtfs_realtime_version: "2.0".into(),
            incrementality: None,
            timestamp: None,
        }
    }
    fn tu(
        ti: &str,
        start_date: &str,
        stu: Vec<StopTimeUpdate>,
        vehicle: Option<VehicleDescriptor>,
    ) -> FeedEntity {
        FeedEntity {
            id: "i".into(),
            is_deleted: None,
            vehicle: None,
            alert: None,
            trip_update: Some(TripUpdate {
                trip: TripDescriptor {
                    trip_id: Some(ti.into()),
                    route_id: None,
                    direction_id: None,
                    start_time: None,
                    start_date: Some(start_date.into()),
                    schedule_relationship: None,
                },
                vehicle,
                stop_time_update: stu,
                timestamp: None,
                delay: None,
            }),
        }
    }
    fn r(start_date: NaiveDate, ti: &str, stop_sequence: u32) -> RealtimeQueryKey {
        RealtimeQueryKey {
            start_date,
            trip_id: ti,
            stop_sequence,
        }
    }
    fn stu_delay(s: u32, delay: Option<i32>, schedule_relationship: Option<i32>) -> StopTimeUpdate {
        StopTimeUpdate {
            stop_sequence: Some(s),
            stop_id: None,
            arrival: None,
            departure: Some(StopTimeEvent {
                delay,
                time: None,
                uncertainty: None,
            }),
            schedule_relationship,
        }
    }
    fn v(id: &str, label: &str) -> VehicleDescriptor {
        VehicleDescriptor {
            id: Some(id.into()),
            label: Some(label.into()),
            license_plate: None,
        }
    }
    #[test]
    fn delays() {
        let feed = FeedMessage {
            header: h(),
            entity: vec![
                tu(
                    "trip1",
                    "20200101",
                    vec![stu_delay(2, Some(20), None), stu_delay(5, Some(-10), None)],
                    None,
                ),
                tu(
                    "trip2",
                    "20200101",
                    vec![stu_delay(1, Some(180), Some(0))],
                    None,
                ),
            ],
        };

        let mut m = RealtimeUpdateManager::new();
        m.load_feed(feed);
        assert_eq!(
            m.get_realtime_updates(vec![r(NaiveDate::from_ymd(2020, 1, 1), "trip1", 1)]),
            vec![RealtimeUpdate {
                delay: None,
                schedule_relationship: None,
                vehicle: None,
            }]
        );
        assert_eq!(
            m.get_realtime_updates(vec![r(NaiveDate::from_ymd(2020, 1, 1), "trip1", 3)]),
            vec![RealtimeUpdate {
                delay: Some(20),
                schedule_relationship: None,
                vehicle: None,
            }]
        );
        assert_eq!(
            m.get_realtime_updates(vec![r(NaiveDate::from_ymd(2020, 1, 1), "trip1", 5)]),
            vec![RealtimeUpdate {
                delay: Some(-10),
                schedule_relationship: None,
                vehicle: None,
            }]
        );

        assert_eq!(
            m.get_realtime_updates(vec![r(NaiveDate::from_ymd(2020, 1, 1), "trip2", 3)]),
            vec![RealtimeUpdate {
                delay: Some(180),
                schedule_relationship: Some(0),
                vehicle: None,
            }]
        );
        // different date
        assert_eq!(
            m.get_realtime_updates(vec![r(NaiveDate::from_ymd(2020, 1, 2), "trip1", 5)]),
            vec![RealtimeUpdate {
                delay: None,
                schedule_relationship: None,
                vehicle: None,
            }]
        );
    }
    #[test]
    fn vehicle_data() {
        let feed = FeedMessage {
            header: h(),
            entity: vec![tu("trip1", "20200101", vec![], Some(v("train1", "AT1345")))],
        };

        let mut m = RealtimeUpdateManager::new();
        m.load_feed(feed);

        assert_eq!(
            m.get_realtime_updates(vec![r(NaiveDate::from_ymd(2020, 1, 1), "trip1", 2)]),
            vec![RealtimeUpdate {
                delay: None,
                schedule_relationship: None,
                vehicle: Some(v("train1", "AT1345")),
            }]
        );
    }
}
