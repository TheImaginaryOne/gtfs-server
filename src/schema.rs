table! {
    agency (feed_id, agency_id) {
        feed_id -> Int4,
        agency_id -> Text,
        agency_name -> Text,
        agency_url -> Text,
        agency_timezone -> Text,
        agency_lang -> Nullable<Text>,
        agency_phone -> Nullable<Text>,
    }
}

table! {
    calendar (feed_id, service_id) {
        feed_id -> Int4,
        service_id -> Text,
        monday -> Bool,
        tuesday -> Bool,
        wednesday -> Bool,
        thursday -> Bool,
        friday -> Bool,
        saturday -> Bool,
        sunday -> Bool,
        start_date -> Date,
        end_date -> Date,
    }
}

table! {
    calendar_date (feed_id, service_id, date) {
        feed_id -> Int4,
        service_id -> Text,
        date -> Date,
        exception_type -> Int4,
    }
}

table! {
    feed (feed_id) {
        feed_id -> Int4,
    }
}

table! {
    realtime_feed (region_id) {
        region_id -> Text,
        url_config -> Text,
    }
}

table! {
    route (feed_id, route_id) {
        feed_id -> Int4,
        route_id -> Text,
        agency_id -> Text,
        route_short_name -> Nullable<Text>,
        route_long_name -> Nullable<Text>,
        route_type -> Int4,
        route_color -> Nullable<Text>,
        route_text_color -> Nullable<Text>,
    }
}

table! {
    shape (feed_id, shape_id, shape_pt_sequence) {
        feed_id -> Int4,
        shape_id -> Text,
        shape_pt_lat -> Float8,
        shape_pt_lon -> Float8,
        shape_pt_sequence -> Int4,
        shape_dist_traveled -> Nullable<Float8>,
    }
}

table! {
    stop (feed_id, stop_id) {
        feed_id -> Int4,
        stop_id -> Text,
        stop_code -> Nullable<Text>,
        stop_name -> Text,
        stop_desc -> Nullable<Text>,
        stop_lat -> Float8,
        stop_lon -> Float8,
        zone_id -> Nullable<Text>,
        parent_station -> Nullable<Text>,
        location_type -> Nullable<Int4>,
    }
}

table! {
    stop_time (feed_id, trip_id, stop_sequence) {
        feed_id -> Int4,
        trip_id -> Text,
        arrival_time -> Int4,
        departure_time -> Int4,
        stop_id -> Text,
        stop_sequence -> Int4,
        stop_headsign -> Nullable<Text>,
        shape_dist_traveled -> Nullable<Int4>,
        pickup_type -> Nullable<Int4>,
        drop_off_type -> Nullable<Int4>,
    }
}

table! {
    trip (feed_id, trip_id) {
        feed_id -> Int4,
        trip_id -> Text,
        route_id -> Text,
        service_id -> Text,
        trip_headsign -> Nullable<Text>,
        trip_short_name -> Nullable<Text>,
        direction_id -> Nullable<Bool>,
        shape_id -> Nullable<Text>,
        block_id -> Nullable<Text>,
    }
}

joinable!(calendar -> feed (feed_id));
joinable!(calendar_date -> feed (feed_id));
joinable!(route -> feed (feed_id));
joinable!(shape -> feed (feed_id));
joinable!(stop -> feed (feed_id));
joinable!(stop_time -> feed (feed_id));

allow_tables_to_appear_in_same_query!(
    agency,
    calendar,
    calendar_date,
    feed,
    realtime_feed,
    route,
    shape,
    stop,
    stop_time,
    trip,
);
