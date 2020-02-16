<template>
    <div class="sidebar">
        <div class="sidebar-container">
            <InputBar @submit="submit" />
        </div>
        <div class="sidebar-container">
            <Timetable />
        </div>
    </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import InputBar from './InputBar.vue'
import Timetable from './Timetable.vue'

@Component({
    components: { Timetable, InputBar }
})
export default class SideBar extends Vue {

    private submit(i: String) {
        console.log(`Inputted ${i}`)
    }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss">
.sidebar-container {
    padding: 0.5rem 1rem;
}
</style>
