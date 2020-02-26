<template>
  <div id="map">
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import mapboxgl from 'mapbox-gl'

@Component
export default class Map extends Vue {
  map!: mapboxgl.Map;

  mounted() {
    this.map = new mapboxgl.Map({
      container: 'map',
      style: 'https://api.maptiler.com/maps/streets/style.json?key=HMG6TdCwKKBPhZzmLSn5',
      center: [0, 0],
      zoom: 3
    })
  }
}
</script>
<!-- Add "scoped" attribute to limit CSS to this component only -->
<style lang="scss">
@import 'node_modules/mapbox-gl/dist/mapbox-gl';

#map {
  height: 100%;
}
</style>
