export interface TimetableUpdate {
  base: BaseStopTime,
  realtime: RealtimeUpdate
}
export interface BaseStopTime {
  stop_id: string,
  trip_id: string,
  departure_time: string,
  service_date: string,
  stop_sequence: number,
  direction_id?: Boolean,
  trip_headsign?: string,
  route_short_name?: string,
  route_long_name?: string,
  route_type: number
}
export interface RealtimeUpdate { 
  delay?: number,
  departure_time?: string,
  schedule_relationship?: number,
  vehicle?: VehicleDescriptor
}
export interface VehicleDescriptor {
  id: string,
  label?: string,
  license_plate?: string
}
