<!-- Copyright (c) 2024 Roc Streaming authors
     Licensed under MPL-2.0 -->

<template>
  <div v-if="show" class="modal-mask">
    <div class="modal-wrapper">
      <div class="modal-container">
        <div>
          <div class="headline">Create stream device</div>
          <div class="property">
            <div class="key">type:</div>
            <select id="device_type">
              <option value="sink">stream sink (output device)</option>
              <option value="source">stream source (input device)</option>
            </select>
          </div>
          <div class="property">
            <div class="key">display name:</div>
            <input type="text" id="display_name" />
          </div>
          <div class="property">
            <div class="key">audio source:</div>
            <input type="text" id="audio_source" />
          </div>
          <div class="property">
            <div class="key">audio repair:</div>
            <input type="text" id="audio_repair" />
          </div>
          <div class="property">
            <div class="key">audio control:</div>
            <input type="text" id="audio_control" />
          </div>
        </div>
        <div class="buttonline">
          <button @click="create">Create device</button>
          <button @click="cancel">Cancel</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-mask {
  position: fixed;
  z-index: 9998;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
}

.modal-container {
  width: 500px;
  margin: 300px auto;
  padding: 20px 30px;
  background-color: #fff;
  border-radius: 2px;
}

.headline {
  font-weight: bold;
  font-size: 18px;
  margin-bottom: 15px;
}

.buttonline {
  display: flex;
  justify-content: flex-end;
  margin-top: 15px;
}

.buttonline > button {
  margin-left: 5px;
}

.property {
  display: flex;
  font-size: 15px;
  margin-bottom: 5px;
}

.property > .key {
  min-width: 150px;
}

.property > input {
  width: 100%;
}
</style>

<script>
import axios from 'axios'

export default {
  props: {
    show: Boolean,
  },
  emits: ['cancel'],
  methods: {
    async create() {
      let type = document.getElementById('device_type').value
      let addr = type == 'sink' ? 'to_address' : 'from_address'
      let data = {
        type: type,
        display_name: document.getElementById('display_name').value,
        [addr]: [
          {
            audio_source: document.getElementById('audio_source').value,
            audio_repair: document.getElementById('audio_repair').value,
            audio_control: document.getElementById('audio_control').value,
          },
        ],
      }
      await axios.post(`/stream_devices`, data)
      this.$emit('close', 0)
    },
    cancel() {
      this.$emit('close', 0)
    },
  },
}
</script>
