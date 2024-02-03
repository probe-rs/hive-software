<script setup lang="ts">
import {
  ResultEnum,
  State,
  type BackendMutation,
  type BackendMutationAssignTargetArgs,
  type BackendQuery,
  type FlatTargetState,
} from "@/gql/backend";

import { ref, watch, type PropType } from "vue";
import { useMutation, useQuery } from "@vue/apollo-composable";
import { gql } from "@apollo/client/core";
import { computed, toRefs } from "vue";
import { cloneDeep } from "@apollo/client/utilities";

const props = defineProps({
  target: {
    type: Number,
    required: true,
  },
  status: {
    type: String as PropType<ResultEnum>,
    required: true,
  },
  statusMessage: {
    type: String,
    required: true,
  },
  tssPos: {
    type: Number,
    required: true,
  },
  initialData: {
    type: Object as PropType<FlatTargetState>,
    required: true,
  },
});

const { target, status, statusMessage, tssPos, initialData } = toRefs(props);

const search = ref("");
const selectedChip = ref(getInitialSelectedChip());

const { result: searchResults, loading: searchLoading } =
  useQuery<BackendQuery>(
    gql`
      query ($search: String) {
        searchSupportedTargets(search: $search)
      }
    `,
    { search },
  );

const { mutate: submitTarget } = useMutation<
  BackendMutation,
  BackendMutationAssignTargetArgs
>(
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
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const assignTarget = data.assignTarget;

      const QUERY = gql`
        query {
          assignedTargets {
            state
            data {
              name
              flashStatus
              flashMessage
            }
          }
        }
      `;

      let cacheData: BackendQuery | null = cache.readQuery({
        query: QUERY,
      });

      if (!cacheData) {
        return;
      }

      let newTarget: FlatTargetState;

      switch (assignTarget.targetName) {
        case "Unknown":
          newTarget = {
            state: State.Unknown,
            data: null,
          };
          break;
        case "Not Connected":
          newTarget = {
            state: State.NotConnected,
            data: null,
          };
          break;
        default:
          newTarget = {
            state: State.Known,
            data: {
              name: assignTarget.targetName,
              flashStatus: ResultEnum.Error,
              flashMessage: "Not initialized yet",
            },
          };
          break;
      }

      const newAssignedTargets = cloneDeep(cacheData.assignedTargets);

      if (!newAssignedTargets) {
        return;
      }

      newAssignedTargets[assignTarget.tssPos]![assignTarget.targetPos] =
        newTarget;

      cacheData = {
        ...cacheData,
        assignedTargets: newAssignedTargets,
      };

      cache.writeQuery<BackendQuery>({ query: QUERY, data: cacheData });
    },
  },
);

function getInitialSelectedChip() {
  switch (initialData.value.state) {
    case State.Unknown:
      return "Unknown";
    case State.NotConnected:
      return "Not Connected";
    default:
      return initialData.value.data!.name;
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

watch(tssPos, () => {
  selectedChip.value = getInitialSelectedChip();
});

function submit(targetName: string) {
  submitTarget({ tssPos: tssPos.value, targetPos: target.value, targetName });
}
</script>

<template>
  <v-card elevation="1">
    <v-card-title prepend-icon="mdi-chip">
      <v-icon icon="mdi-chip" size="40" class="mr-2" />
      Target {{ target }}
    </v-card-title>

    <template v-if="initialData.state === State.Known">
      <v-card-subtitle v-if="status === ResultEnum.Error">
        <v-icon icon="mdi-alert" size="18" color="error" class="mr-1 pb-1" />

        {{ statusMessage }}
      </v-card-subtitle>

      <v-card-subtitle v-else>
        <v-icon
          icon="mdi-checkbox-marked"
          size="18"
          color="success"
          class="mr-1 pb-1"
        />

        {{ statusMessage }}
      </v-card-subtitle>
    </template>

    <template v-else>
      <v-card-subtitle style="visibility: hidden">
        <v-icon
          icon="mdi-checkbox-marked"
          size="18"
          color="success"
          class="mr-1 pb-1"
        />

        placeholder
      </v-card-subtitle>
    </template>

    <v-card-text class="pb-0" style="margin-bottom: 1vh">
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
