<script setup lang="ts">
import Testrack from "@/components/testrack/Testrack.vue";
import TssOverview from "@/components/TssOverview.vue";
import { ref } from "vue";

const selectedPartLocation = ref(-1);

function handleSelect(location: number) {
  selectedPartLocation.value = location;
}
</script>

<template>
  <v-row dense>
    <Testrack @selectedPartLocation="handleSelect" />
  </v-row>
  <template v-if="selectedPartLocation > 1">
    <TssOverview :tssPos="selectedPartLocation - 2" />
  </template>
  <template v-else-if="selectedPartLocation === 1">
    <p>PSS selected</p>
  </template>
  <template v-else-if="selectedPartLocation === 0">
    <p>RPI selected</p>
  </template>
  <template v-else>
    <p>Nothing selected</p>
  </template>
</template>
