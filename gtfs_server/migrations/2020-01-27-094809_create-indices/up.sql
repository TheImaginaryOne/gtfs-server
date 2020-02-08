-- Your SQL goes here
create index index_stop_time_stop_id on stop_time(stop_id);
create index index_stop_stop_code on stop(stop_code);

CREATE INDEX trip_trip_id_idx ON trip (trip_id);
CREATE INDEX calendar_date_service_id_idx ON public.calendar_date (service_id);
