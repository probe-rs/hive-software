<script setup lang="ts">
import Testrack from "@/components/testrack/Testrack.vue";
import TssOverview from "@/components/TssOverview.vue";
import PssOverview from "@/components/PssOverview.vue";
import RpiOverview from "@/components/RpiOverview.vue";
import { ref } from "vue";

const selectedPartLocation = ref(-1);

function handleSelect(location: number) {
  selectedPartLocation.value = location;
}
</script>

<template>
  <v-row dense justify="center">
    <Testrack @selectedPartLocation="handleSelect" />
  </v-row>
  <template v-if="selectedPartLocation > 1">
    <TssOverview :tssPos="selectedPartLocation - 2" />
  </template>
  <template v-else-if="selectedPartLocation === 1">
    <PssOverview />
  </template>
  <template v-else-if="selectedPartLocation === 0">
    <RpiOverview />
  </template>
  <template v-else>
    <v-row justify="center">
      <v-col cols="12" class="pa-4">
        <v-row class="justify-center">
          <p class="align-self-center" style="
              max-width: 70%;
              text-align: center;
              color: rgb(var(--v-theme-on-surface), var(--v-disabled-opacity));
            ">
            Select a part in the Testrack above to display information and
            adjust settings of this part
          </p>
        </v-row>
      </v-col>
    </v-row>
  </template>
</template>
