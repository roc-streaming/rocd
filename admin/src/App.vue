<!-- Copyright (c) 2024 Roc Streaming authors
     Licensed under MPL-2.0 -->

<script setup>
import DeviceBlock from './DeviceBlock.vue'
import DeviceCreate from './DeviceCreate.vue'
</script>

<template>
  <div id="app">
    <div class="caption">Devices</div>
    <DeviceBlock v-for="dev in systemDevices" :key="dev.device_uid" :device="dev" />

    <div class="caption">Stream devices</div>
    <DeviceBlock v-for="dev in streamDevices" :key="dev.device_uid" :device="dev" />
    <button @click="raiseStreamDevice">Create stream device</button>

    <DeviceCreate :show="showDeviceCreate" @close="hideStreamDevice" />
  </div>
</template>

<style scoped>
#app {
  max-width: 800px;
  margin: 10px;
}

.caption {
  padding-top: 20px;
  padding-bottom: 10px;
  font-size: 20px;
  font-weight: bold;
}
</style>

<script>
import axios from 'axios'

export default {
  name: 'App',
  data() {
    return {
      devices: [],
      showDeviceCreate: false,
    }
  },
  computed: {
    systemDevices() {
      return this.devices.filter((dev) => !dev.stream_device)
    },
    streamDevices() {
      return this.devices.filter((dev) => dev.stream_device)
    },
  },
  methods: {
    async processEvent(event) {
      const response = await axios.get('/devices')
      this.devices = response.data
    },
    raiseStreamDevice() {
      this.showDeviceCreate = true
    },
    hideStreamDevice() {
      this.showDeviceCreate = false
    },
  },
  created() {
    this.eventSource = new EventSource('/events')
    this.eventSource.onmessage = this.processEvent
  },
  beforeUnmount() {
    this.eventSource.close()
  },
}
</script>
