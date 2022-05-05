<script setup lang="ts">
import Testrack from "@/components/testrack/Testrack.vue";
import TargetOverview from "@/components/TargetOverview.vue";
</script>

<template>
  <v-row dense>
    <Testrack />
  </v-row>
  <v-row>
    <v-col cols="6">
      <TargetOverview :target="0" :status="false" />
    </v-col>
    <v-col cols="6">
      <TargetOverview :target="1" :status="true" />
    </v-col>
  </v-row>
  <v-row>
    <v-col cols="6">
      <TargetOverview :target="2" :status="true" />
    </v-col>
    <v-col cols="6">
      <TargetOverview :target="3" :status="true" />
    </v-col>
  </v-row>
</template>
