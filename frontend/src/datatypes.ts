export interface TimetableUpdate {
    base: BaseStopTime,
    realtime: RealtimeUpdate
}
export interface BaseStopTime {
    stop_id: String,
    trip_id: String,
    departure_time: String,
    service_date: String,
    stop_sequence: Number,
    direction_id?: Boolean,
    trip_headsign?: String,
    route_short_name?: String,
    route_long_name?: String,
    route_type: Number
}
export interface RealtimeUpdate { 
    delay?: Number,
    schedule_relationship?: Number,
    vehicle?: VehicleDescriptor
}
export interface VehicleDescriptor {
    id: String,
    label?: String,
    license_plate?: String
}
