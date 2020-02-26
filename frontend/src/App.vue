<template>
  <div id="app">
    <div id="container">
      <div id="sidebar-wrapper">
        <SideBar/>
      </div>
      <div id="main-wrapper">
        <Map/>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import SideBar from './components/SideBar.vue'
import Map from './components/Map.vue'

@Component({
  components: {
    SideBar,
    Map
  }
})
export default class App extends Vue {
}
</script>

<style lang="scss">
* {
  box-sizing: border-box;
}
body, html {
  margin: 0;
  padding: 0;
  height: 100%;
}
#app {
  font-family: "Roboto", Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  margin: 0;
  height: 100%;
}
#container {
  display: flex;
  height: 100%;
}
#sidebar-wrapper {
  flex-basis: 600px;
}
#main-wrapper {
  flex-grow: 1;
}
</style>
