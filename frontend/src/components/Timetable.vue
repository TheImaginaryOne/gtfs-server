<template>
  <div class="timetable">
    <div v-for="(update, i) in timetableData" :key="update.base.trip_id + ':' + update.base.service_date">
      <div class="timetable-update-row">
        <div class="route-info-column">
          <div class="route-info">
            <div class="short-name-pill">{{update.base.route_short_name}}</div>
            <div class="trip-headsign">{{update.base.trip_headsign}}</div>
          </div>
          <div class="route-additional-info">
            <div class="route-vehicle" v-if="update.realtime !== null && update.realtime.vehicle !== null">
              Vehicle: {{update.realtime.vehicle.label}} </div>
          </div>
        </div>
        <div class="route-time-column">
          <div class="route-time">{{ departureTimeMinutes[i] }} min</div>
          <div class="time-additional-info">
            <span v-if="update.realtime !== null && update.realtime.delay !== null">
              Delayed {{Math.floor(update.realtime.delay / 60)}} min
            </span>
            <span v-else>
              Scheduled
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import { TimetableUpdate } from '../datatypes'
import moment from 'moment'

@Component({
  components: {}
})
export default class Timetable extends Vue {
  @Prop()
  private timetableData?: TimetableUpdate[]

  @Prop()
  private error?: string

  get departureTimeMinutes(): number[] | undefined {
    return this.timetableData?.map(x => {
      let minutes = moment.duration(
        moment(x.realtime? x.realtime.departure_time: x.base.departure_time).diff(moment())
      ).asMinutes()
      if (minutes > 0) {
        return Math.floor(minutes);
      } else {
        return Math.ceil(minutes);
      }
    })
  }
}
</script>
<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss">
.timetable-update-row {
  border-bottom: 1px solid #ddd;
  display: flex;
  padding: 12px 0;
}
.short-name-pill {
  border: 1px solid #999;
  border-radius: 4px;
  padding: 4px 8px;
  margin-right: 8px;
  font-weight: bold;
}
.route-info-column {
  width: 320px;
}
.route-info {
  display: flex;
  align-items: center;
}
.trip-headsign {
  font-size: 24px;
  margin-bottom: 2px;
}
.route-additional-info {
  padding-top: 4px;
  display: flex;
}
.route-vehicle {
  color: #777;
}

.route-time-column {
  padding: 2px 0;
  padding-left: 16px;
}
.route-time {
  font-size: 20px;
}
.time-additional-info {
  padding-top: 4px;
  color: #777;
}
</style>
