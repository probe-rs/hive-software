<script setup lang="ts">
import { ref, defineProps, watch } from "vue";
import { useMutation, useQuery } from "@vue/apollo-composable";
import { gql } from "@apollo/client/core";
import { computed } from "@vue/reactivity";
import { cloneDeep } from "@apollo/client/utilities";

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
  initialData: {
    type: Object,
    required: true,
  },
});

const search = ref("");
const selectedChip = ref(getInitialSelectedChip());

const { result: searchResults, loading: searchLoading } = useQuery(
  gql`
    query ($search: String) {
      searchSupportedTargets(search: $search)
    }
  `,
  { search },
);

const { mutate: submitTarget } = useMutation(
  gql`
    mutation ($tssPos: Int!, $targetPos: Int!, $targetName: String!) {
      assignTarget(
        tssPos: $tssPos
        targetPos: $targetPos
        targetName: $targetName
      ) {
        tssPos
        targetPos
        targetName
      }
    }
  `,
  {
    update: (cache, { data: { assignTarget } }) => {
      const QUERY = gql`
        query {
          assignedTargets {
            state
            data {
              name
            }
          }
        }
      `;

      let data: any = cache.readQuery({
        query: QUERY,
      });

      let newTarget;

      switch (assignTarget.targetName) {
        case "Unknown":
          newTarget = {
            state: "UNKNOWN",
            data: null,
            __typename: "FlatTargetState",
          };
          break;
        case "Not Connected":
          newTarget = {
            state: "NOT_CONNECTED",
            data: null,
            __typename: "FlatTargetState",
          };
          break;
        default:
          newTarget = {
            state: "KNOWN",
            data: {
              name: assignTarget.targetName,
              __typename: "TargetInfo",
            },
            __typename: "FlatTargetState",
          };
          break;
      }

      const newAssignedTargets = cloneDeep(data.assignedTargets);

      newAssignedTargets[assignTarget.tssPos][assignTarget.targetPos] =
        newTarget;

      data = {
        ...data,
        assignedTargets: newAssignedTargets,
      };

      cache.writeQuery({ query: QUERY, data });
    },
  },
);

function getInitialSelectedChip() {
  if (props.initialData.state === "UNKNOWN") {
    return "Unknown";
  } else if (props.initialData.state === "NOT_CONNECTED") {
    return "Not Connected";
  } else {
    return props.initialData.data.name;
  }
}

const foundChips = computed(() => {
  if (searchResults.value) {
    const array = searchResults.value.searchSupportedTargets.map(
      (x: string) => x,
    );
    array.push("Unknown", "Not Connected");
    return array;
  }
  return ["Unknown", "Not Connected"];
});

watch(
  () => props.tssPos,
  () => {
    selectedChip.value = getInitialSelectedChip();
  },
);

function submit(targetName: string) {
  submitTarget({ tssPos: props.tssPos, targetPos: props.target, targetName });
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
        @update:modelValue="submit"
        v-model:search="search"
        v-model="selectedChip"
        :loading="searchLoading"
        :items="foundChips"
        dense
        label="Chip model"
        hint="Please select the appropriate chip"
        persistent-hint
        no-data-text="No matching models found"
      >
      </v-autocomplete>
    </v-card-text>
  </v-card>
</template>
