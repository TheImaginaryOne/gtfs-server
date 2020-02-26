<template>
  <div class="sidebar">
    <div class="sidebar-container">
      <InputBar @submit="submit" />
    </div>
    <div class="sidebar-container-timetable">
      <Timetable :timetableData="timetableData" />
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import InputBar from './InputBar.vue'
import Timetable from './Timetable.vue'
import { TimetableUpdate } from '../datatypes'

// TODO make the url configurable!
async function fetchTimetable(stopId: string): Promise<TimetableUpdate[] | null> {
  const url = `api/stop/${stopId}/times`
  try {
    const response = await fetch(url)
    const data = await response.json()
    return data.trips
  } catch {
    return null
  }
}

@Component({
  components: { Timetable, InputBar }
})
export default class SideBar extends Vue {
  timetableData: TimetableUpdate[] = []

  private async submit(i: string) {
    const timetable = await fetchTimetable(i)
    if (timetable != null) {
      this.timetableData = timetable
    }
  }
}
</script>
<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss">
.sidebar {
  display: flex;
  flex-direction: column;
  max-height: 100%;
}
.sidebar-container {
  padding: 8px 16px;
}
.sidebar-container-timetable {
  padding: 0 16px;
  max-height: 100%;
  overflow-y: scroll;
}
</style>
