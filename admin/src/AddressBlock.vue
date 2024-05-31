<!-- Copyright (c) 2024 Roc Streaming authors
     Licensed under MPL-2.0 -->

<template>
  <div class="address">
    <div class="headline">{{ title }}:</div>
    <div class="property">
      <div class="key">audio source:</div>
      <div class="value">{{ address.audio_source }}</div>
    </div>
    <div class="property">
      <div class="key">audio repair:</div>
      <div class="value">{{ address.audio_repair }}</div>
    </div>
    <div class="property">
      <div class="key">audio control:</div>
      <div class="value">{{ address.audio_control }}</div>
    </div>
  </div>
</template>

<style scoped>
.headline {
  font-weight: bold;
  font-size: 12px;
}

.property {
  display: flex;
  font-size: 13px;
  margin-left: 10px;
}

.property > .key {
  min-width: 210px;
}
</style>

<script>
export default {
  props: {
    title: String,
    address: Object,
  },
}
</script>
