-- Your SQL goes here
create table realtime_feed(
    region_id text primary key not null,
    url_config text not null -- possibly toml syntax, data does not need to be normalised.
);