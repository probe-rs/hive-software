<script setup lang="ts">
import { ref, defineProps } from "vue";

const props = defineProps({
  target: Number,
  status: Boolean,
});

const targetRegistry = ["STM32", "nRF5282", "atmel", "LPC1523"];

const loading = ref(false);

function setDummyLoading() {
  loading.value = true;
  setTimeout(() => {
    loading.value = false;
  }, 2000);
}
</script>

<template>
  <v-card elevation="1">
    <v-card-title prepend-icon="mdi-chip">
      <v-icon icon="mdi-chip" size="40" class="mr-2" />
      Target {{ props.target }}
    </v-card-title>

    <v-card-subtitle v-show="!props.status">
      <v-icon icon="mdi-alert" size="18" color="error" class="mr-1 pb-1" />

      Failed to flash testprogram
    </v-card-subtitle>

    <v-card-subtitle v-show="props.status">
      <v-icon
        icon="mdi-checkbox-marked"
        size="18"
        color="success"
        class="mr-1 pb-1"
      />

      No problems found
    </v-card-subtitle>

    <v-card-text class="pb-0">
      <v-autocomplete
        :items="targetRegistry"
        dense
        label="Chip model"
        hint="Please select the appropriate chip"
        persistent-hint
        no-data-text="No matching models found"
      ></v-autocomplete>
    </v-card-text>

    <v-card-actions>
      <v-btn color="warning" @click="setDummyLoading">Update</v-btn>
    </v-card-actions>

    <v-overlay
      v-model="loading"
      persistent
      contained
      class="align-center justify-center"
    >
      <v-progress-circular color="secondary" size="80" indeterminate />
    </v-overlay>
  </v-card>
</template>
