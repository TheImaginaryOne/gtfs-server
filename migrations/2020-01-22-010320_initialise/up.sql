-- Your SQL goes here

CREATE TABLE feed(
  feed_id serial PRIMARY KEY
);

CREATE TABLE shape (
  feed_id int not null,
  shape_id text not null,
  shape_pt_lat double precision not null,
  shape_pt_lon double precision not null,
  shape_pt_sequence int not null,
  shape_dist_traveled double precision default null,
  CONSTRAINT shape_id PRIMARY KEY (feed_id, shape_id, shape_pt_sequence),
  CONSTRAINT shape_feed_fk FOREIGN KEY (feed_id) references feed(feed_id)
);

CREATE TABLE agency(
  feed_id integer NOT NULL,
  agency_id text NOT NULL,
  agency_name       text NOT NULL,
  agency_url        text NOT NULL,
  agency_timezone   text NOT NULL,
  agency_lang       text NULL,
  agency_phone      text NULL,
  CONSTRAINT agency_pk PRIMARY KEY (feed_id, agency_id),
  CONSTRAINT agency_feed_fk FOREIGN KEY (feed_id) references feed(feed_id)
);

CREATE TABLE route(
  feed_id integer NOT NULL,
  route_id          text NOT NULL,
  agency_id         text NOT NULL,
  route_short_name  text NULL,
  route_long_name   text NULL,
  route_type        integer NOT NULL,
  route_color       text NULL,
  route_text_color  text NULL,
  CONSTRAINT route_pk PRIMARY KEY (feed_id, route_id),
  CONSTRAINT route_feed_fk FOREIGN KEY (feed_id) references feed(feed_id),
  CONSTRAINT route_agency_fk FOREIGN KEY (feed_id, agency_id) references agency(feed_id, agency_id)
);

CREATE TABLE trip(
  feed_id integer NOT NULL,
  trip_id           text NOT NULL,
  route_id          text NOT NULL,
  service_id        text NOT NULL,
  trip_headsign     text NULL,
  trip_short_name   text NULL,
  direction_id      boolean NULL,
  shape_id          text NULL, -- useless for my use case
  block_id          text NULL,
  CONSTRAINT trip_pk PRIMARY KEY (feed_id, trip_id),
  CONSTRAINT agency_feed_fk FOREIGN KEY (feed_id) references feed(feed_id),
  CONSTRAINT trip_route_fk FOREIGN KEY (feed_id, route_id) references route(feed_id, route_id)
);

CREATE TABLE calendar(
  feed_id integer NOT NULL,
  service_id        text NOT NULL,
  monday            boolean NOT NULL,
  tuesday           boolean NOT NULL,
  wednesday         boolean NOT NULL,
  thursday          boolean NOT NULL,
  friday            boolean NOT NULL,
  saturday          boolean NOT NULL,
  sunday            boolean NOT NULL,
  start_date        date NOT NULL,
  end_date          date NOT NULL,
  CONSTRAINT calendar_pk PRIMARY KEY (feed_id, service_id),
  CONSTRAINT calendar_feed_fk FOREIGN KEY (feed_id) references feed(feed_id)
);

CREATE TABLE calendar_date(
  feed_id integer NOT NULL,
  service_id text NOT NULL,
  date       date NOT NULL,
  exception_type integer NOT NULL,
  CONSTRAINT calendar_date_pk PRIMARY KEY (feed_id, service_id, date),
  CONSTRAINT calendar_date_feed_fk FOREIGN KEY (feed_id) references feed(feed_id)
);

CREATE TABLE stop(
  feed_id integer NOT NULL,
  stop_id           text NOT NULL,
  stop_code         text NULL,
  stop_name         text NOT NULL,
  stop_desc         text NULL,
  stop_lat          double precision NOT NULL,
  stop_lon          double precision NOT NULL,
  zone_id           text NULL,
  parent_station    text NULL, -- self references stop_id
  location_type     integer NULL,
  CONSTRAINT stop_pk PRIMARY KEY (feed_id, stop_id),
  CONSTRAINT stop_feed_fk FOREIGN KEY (feed_id) references feed(feed_id)
);

CREATE TABLE stop_time(
  feed_id integer NOT NULL,
  trip_id           text NOT NULL,
  arrival_time      integer NOT NULL, -- in seconds after midnight of service day
  departure_time    integer NOT NULL,
  stop_id           text NOT NULL,
  stop_sequence     integer NOT NULL,
  stop_headsign     text NULL,
  shape_dist_traveled integer NULL,
  pickup_type       integer NULL,
  drop_off_type     integer NULL,
  CONSTRAINT stop_time_pk PRIMARY KEY (feed_id, trip_id, stop_sequence),
  CONSTRAINT stop_time_trip_fk FOREIGN KEY (feed_id, trip_id) references trip(feed_id, trip_id),
  CONSTRAINT stop_time_stop_fk FOREIGN KEY (feed_id, stop_id) references stop(feed_id, stop_id),
  CONSTRAINT stop_time_feed_fk FOREIGN KEY (feed_id) references feed(feed_id)
);