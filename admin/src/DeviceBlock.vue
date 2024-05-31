<!-- Copyright (c) 2024 Roc Streaming authors
     Licensed under MPL-2.0 -->

<script setup>
import AddressBlock from './AddressBlock.vue'
</script>

<template>
  <div class="device">
    <div class="headline">
      <div>{{ device.display_name }}</div>
      <div v-if="!device.stream_device">{{ device.type }}</div>
      <div v-if="device.stream_device">stream {{ device.type }}</div>
    </div>
    <div class="property">
      <div class="key">uid:</div>
      <div class="value">{{ device.device_uid }}</div>
    </div>
    <div class="property">
      <div class="key">system name:</div>
      <div class="value">{{ device.system_name }}</div>
    </div>
    <div class="property">
      <div class="key">driver:</div>
      <div class="value">{{ device.driver }}</div>
    </div>
    <div class="property">
      <div class="key">flags:</div>
      <div class="value">{{ deviceFlags }}</div>
    </div>
    <AddressBlock
      v-for="addr in device.to_address"
      :key="addr.source_address"
      :title="'to address'"
      :address="addr"
    />
    <AddressBlock
      v-for="addr in device.from_address"
      :key="addr.source_address"
      :title="'from address'"
      :address="addr"
    />
    <div class="buttonline">
      <button v-if="device.stream_device" @click="deleteDevice">Delete</button>
      <button
        v-if="device.stream_device && device.status == 'disabled'"
        @click="enableDevice"
      >
        Enable
      </button>
      <button
        v-if="device.stream_device && device.status != 'disabled'"
        @click="disableDevice"
      >
        Disable
      </button>
      <button v-if="device.muted" @click="unmuteDevice">Unmute</button>
      <button v-if="!device.muted" @click="muteDevice">Mute</button>
    </div>
  </div>
</template>

<style scoped>
.device {
  border: 1px solid #aaaaaa;
  border-radius: 5px;
  padding: 5px;
  margin-bottom: 8px;
}

.headline {
  display: flex;
  justify-content: space-between;
  font-weight: bold;
  margin-bottom: 5px;
  padding-right: 5px;
  background: #eeeeee;
  font-size: 14px;
}

.buttonline {
  display: flex;
  justify-content: flex-end;
  margin-top: 5px;
  font-size: 13px;
}

.buttonline > button {
  width: 80px;
  margin-right: 5px;
}

.property {
  display: flex;
  font-size: 13px;
}

.property > .key {
  min-width: 200px;
}
</style>

<script>
import axios from 'axios'

export default {
  props: {
    device: Object,
  },
  computed: {
    deviceFlags() {
      let flags = ''
      flags += this.device.hardware_device ? 'hardware ' : 'virtual '
      flags += this.device.status + ' '
      flags += this.device.muted ? 'muted' : 'unmuted'
      return flags
    },
  },
  methods: {
    async muteDevice() {
      await axios.put(`/devices/${this.device.device_uid}`, { muted: true })
    },
    async unmuteDevice() {
      await axios.put(`/devices/${this.device.device_uid}`, { muted: false })
    },
    async enableDevice() {
      await axios.put(`/devices/${this.device.device_uid}`, { status: 'enabled' })
    },
    async disableDevice() {
      await axios.put(`/devices/${this.device.device_uid}`, { status: 'disabled' })
    },
    async deleteDevice() {
      await axios.delete(`/stream_devices/${this.device.device_uid}`)
    },
  },
}
</script>
