with sd as materialized (
	-- service_date_midnight means a timestamp 00:00 on a service date (local time)
	select a.agency_id, series.t service_date_midnight from agency a
		join lateral (
			-- agency_timezone is a timezone string (like Pacific/Auckland)
			-- date_trunc changes the time of the timestamp to 00:00
			select t from generate_series(date_trunc('day', ($1 - '24 hours'::interval) at time zone a.agency_timezone) at time zone a.agency_timezone,
			$2, '1 day'::interval) as t
		) as series on true
), y as materialized (
	select st.stop_id,
		st.trip_id,
		route.route_short_name,
		route.route_long_name,
		route.route_type,
		(st.departure_time * '1 second'::interval + sd.service_date_midnight) as departure_time,
		(sd.service_date_midnight at time zone agency.agency_timezone)::date as service_date,
		trip.service_id as service_id,
		trip.direction_id,
		trip.trip_headsign,
		st.stop_sequence,
		st.feed_id
	from stop_time st
	join stop on st.stop_id = stop.stop_id and stop.stop_code = $3 and st.feed_id = stop.feed_id
	join trip on st.trip_id = trip.trip_id and st.feed_id = trip.feed_id
	join route on trip.route_id = route.route_id and trip.feed_id = route.feed_id
	join agency on route.agency_id = agency.agency_id and route.feed_id = agency.feed_id
	join sd
	 	on route.agency_id = sd.agency_id
		and st.departure_time >= extract (epoch from ($1 - sd.service_date_midnight))
		and st.departure_time <= extract (epoch from ($2 - sd.service_date_midnight))
	--where st.stop_id = '0133-20191217130301_v86.30' or st.stop_id = '0116-20191205152914_v86.28'
	--where st.stop_id = '0116-20191217130301_v86.30' or st.stop_id = '0116-20191205152914_v86.28'
	where (st.pickup_type is null or st.pickup_type != 1)
)
(
select
	y.stop_id,
	y.trip_id,
	y.departure_time,
	y.service_date,
	y.stop_sequence,
	y.direction_id,
	y.trip_headsign,
	y.route_short_name,
	y.route_long_name,
	y.route_type from y
left join calendar_date cd on (y.service_date = cd.date and y.service_id = cd.service_id) and y.feed_id = cd.feed_id
left join calendar cal on y.service_id = cal.service_id and y.feed_id = cal.feed_id
	 where (cd.exception_type is null or cd.exception_type != 2)
		and
		(
			case (extract (dow from y.service_date))
				when 1 then cal.monday
				when 2 then cal.tuesday
				when 3 then cal.wednesday
				when 4 then cal.thursday
				when 5 then cal.friday
				when 6 then cal.saturday
			else cal.sunday
			end
			or cd.exception_type = 1
		) and cal.start_date <= y.service_date and cal.end_date >= y.service_date
)
order by y.departure_time asc
