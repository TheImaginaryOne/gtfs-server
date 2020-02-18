<template>
  <div id="app">
    <div id="container">
      <SideBar/>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import SideBar from './components/SideBar.vue'

@Component({
  components: {
    SideBar
  }
})
export default class App extends Vue {
}
</script>

<style lang="scss">
body {
  margin: 0;
  padding: 0;
}
#app {
  font-family: "Roboto", Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  margin: 0;
}
</style>
