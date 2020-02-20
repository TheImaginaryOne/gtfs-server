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
              Vehicle: {{update.realtime.vehicle.label}}
            </div>
          </div>
        </div>
        <div class="route-time-column">
          <StopTimeUpdate
            :dueTime="computedData[i].dueTime"
            :stopTime="computedData[i].departureTime"
            :delay="update.realtime !== null ? update.realtime.delay : null"/>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import { TimetableUpdate } from '../datatypes'
import moment from 'moment'
import StopTimeUpdate from './StopTimeUpdate.vue'

interface ComputedData {
  dueTime: number;
  departureTime: moment.Moment;
}

@Component({
  components: { StopTimeUpdate }
})
export default class Timetable extends Vue {
  @Prop()
  private timetableData!: TimetableUpdate[]

  @Prop()
  private error?: string

  get computedData(): ComputedData[] {
    return this.timetableData.map(x => {
      const departureTime = moment(x.realtime !== null ? x.realtime.departure_time : x.base.departure_time)
      const dueTime = moment.duration(
        departureTime.diff(moment())
      ).asSeconds()

      return {
        dueTime: dueTime,
        departureTime: departureTime
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
  color: #777;
}
.route-vehicle {
}

.route-time-column {
  padding: 2px 0;
  padding-left: 16px;
}
</style>
