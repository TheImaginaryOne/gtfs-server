<template>
<div>
<div class="route-time">{{ dueText }}</div>
<div class="time-additional-info">
  <span :class="delayStyle.cls">
    {{ delayStyle.text }}
  </span>
</div>
</div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator'
import moment from 'moment'

interface DelayStyle {
  text: string;
  cls: string;
}

@Component
export default class StopTimeUpdate extends Vue {
  @Prop() stopTime!: moment.Moment
  @Prop() dueTime!: number
  @Prop() delay?: number

  get dueText(): string {
    if (this.dueMinutes === 0) {
      return 'Now'
    } else if (this.dueMinutes < 30) {
      return `${this.dueMinutes} min`
    } else {
      return this.stopTime.format('HH:mm')
    }
  }

  get dueMinutes(): number {
    return Math.round(this.dueTime / 60.0)
  }

  get delayStyle(): DelayStyle {
    const style = this.delayStyleBase()
    if (this.dueMinutes < 30) {
      style.text += ` - ${this.stopTime.format('HH:mm')}`
    }
    return style
  }

  delayStyleBase(): DelayStyle {
    const delayMinutes = this.delay ? Math.round(this.delay / 60.0) : null
    if (delayMinutes !== null) {
      if (delayMinutes > 0) {
        return {
          text: `Delayed ${delayMinutes} min`,
          cls: 'text-warning'
        }
      } else if (delayMinutes < 0) {
        return {
          text: `Early ${-delayMinutes} min`,
          cls: 'text-ok'
        }
      } else {
        return {
          text: 'On time',
          cls: 'text-ok'
        }
      }
    } else {
      return {
        text: 'Scheduled',
        cls: 'text-disabled'
      }
    }
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped lang="scss">
.route-time {
  font-size: 20px;
}
.time-additional-info {
  padding-top: 4px;
}
/* todo move */
.text-ok {
  color: #27ae60;
}
.text-disabled {
  color: #777;
}
.text-warning {
  color: #e67e22;
}
</style>
