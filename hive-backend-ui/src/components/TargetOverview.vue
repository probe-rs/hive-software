<script setup lang="ts">
import { ref, defineProps, watch } from "vue";
import { useQuery } from '@vue/apollo-composable'
import { gql } from "@apollo/client/core";
import { computed } from "@vue/reactivity";

const props = defineProps({
  target: {
    type: Number,
    required: true,
  },
  status: Boolean,
  tssPos: {
    type: Number,
    required: true,
  },
});

const search = ref("");
const selectedChip = ref("");

const { result, loading } = useQuery(gql`
    query ($search: String) {
      searchSupportedTargets(search: $search)
    }
  `, { search });

const foundChips = computed(() => {
  if (result.value) {
    const array = result.value.searchSupportedTargets.map((x: String) => x);
    array.push("Unknown", "Not Connected")
    return array;
  }
  return ["Unknown", "Not Connected"];
})

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
      <v-icon icon="mdi-checkbox-marked" size="18" color="success" class="mr-1 pb-1" />

      No problems found
    </v-card-subtitle>

    <v-card-text class="pb-0">
      <v-autocomplete v-model:search="search" v-model="selectedChip" :loading="loading" :items="foundChips" dense
        label="Chip model" hint="Please select the appropriate chip" persistent-hint
        no-data-text="No matching models found">
      </v-autocomplete>
    </v-card-text>
  </v-card>
</template>
