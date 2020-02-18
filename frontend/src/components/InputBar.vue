<template>
  <div class="input-form">
    <form @submit="submit">
      <input v-model="textBoxInput"/>
    </form>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'

@Component
export default class InputBar extends Vue {
  textBoxInput = ''

  private submit(e: Event) {
    this.$emit('submit', this.textBoxInput)
    e.preventDefault()
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss">
input {
  padding: 0.5rem;
  border: 1px solid #ccc;
}
</style>
