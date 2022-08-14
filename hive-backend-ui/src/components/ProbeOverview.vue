<script setup lang="ts">
import {
  type BackendMutation,
  type BackendMutationAssignProbeArgs,
  type BackendQuery,
  State,
  type FlatProbeState,
  type ProbeInfo,
} from "@/gql/backend";

import { cloneDeep } from "@apollo/client/utilities";
import { useMutation, useQuery } from "@vue/apollo-composable";
import gql from "graphql-tag";
import { computed, ref, type PropType } from "vue";

const props = defineProps({
  channel: {
    type: Number,
    required: true,
  },
  initialData: {
    type: Object as PropType<FlatProbeState>,
    required: true,
  },
});

const selectedProbe = ref(displayAssignedProbe(props.initialData));

const { loading, result } = useQuery<BackendQuery>(gql`
  query {
    assignedProbes {
      state
      data {
        identifier
        serialNumber
      }
    }
    connectedProbes {
      identifier
      serialNumber
    }
  }
`);

const { mutate: submitProbe } = useMutation<
  BackendMutation,
  BackendMutationAssignProbeArgs
>(
  gql`
    mutation ($probePos: Int!, $probeState: FlatProbeStateInput!) {
      assignProbe(probePos: $probePos, probeState: $probeState) {
        probePos
        data {
          state
          data {
            identifier
            serialNumber
          }
        }
      }
    }
  `,
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const assignProbe = data.assignProbe;

      const QUERY = gql`
        query {
          assignedProbes {
            state
            data {
              identifier
              serialNumber
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

      const newAssignedProbes = cloneDeep(cacheData.assignedProbes);

      newAssignedProbes[assignProbe.probePos] = assignProbe.data;

      cacheData = {
        ...cacheData,
        assignedProbes: newAssignedProbes,
      };

      cache.writeQuery<BackendQuery>({ query: QUERY, data: cacheData });
    },
  },
);

const assignedProbes = computed(() => {
  if (result.value) {
    return result.value.assignedProbes;
  }
  return [];
});

const connectedProbes = computed(() => {
  if (result.value) {
    return result.value.connectedProbes;
  }
  return [];
});

const availableProbes = computed(() => {
  const available = connectedProbes.value.filter(
    (connectedProbe: ProbeInfo) => {
      for (let i = 0; i < assignedProbes.value.length; i++) {
        if (assignedProbes.value[i].state !== State.Known) {
          continue;
        }

        if (
          assignedProbes.value[i].data!.identifier ===
          connectedProbe.identifier &&
          assignedProbes.value[i].data!.serialNumber ===
          connectedProbe.serialNumber
        ) {
          return false;
        }
      }
      return true;
    },
  );

  const displayAvailable = available.map((availableProbe: ProbeInfo) => {
    return displayProbe(availableProbe);
  });

  displayAvailable.push("Unknown", "Not Connected");

  return displayAvailable;
});

function displayAssignedProbe(probe: FlatProbeState): string {
  switch (probe.state) {
    case State.Known:
      return `${probe.data!.identifier} (S/N: ${probe.data!.serialNumber ? probe.data!.serialNumber : "unknown"
        })`;
    case State.Unknown:
      return "Unknown";
    case State.NotConnected:
      return "Not Connected";
    default:
      return `Received unknown state: ${probe.state}`;
  }
}

function displayProbe(probe: ProbeInfo): string {
  return `${probe.identifier} (S/N: ${probe.serialNumber ? probe.serialNumber : "unknown"
    })`;
}

function submit(probeName: string) {
  let probeState;

  switch (probeName) {
    case "Unknown":
      probeState = {
        state: State.Unknown,
        data: null,
      };
      break;
    case "Not Connected":
      probeState = {
        state: State.NotConnected,
        data: null,
      };
      break;
    default: {
      const findIdentifierRegex = /.+?(?= \(S\/N:)/;
      const findSerialNumberRegex = /\(S\/N: (.*?)\)/;

      const identifierMatches = probeName.match(findIdentifierRegex);
      const serialNumberMatches = probeName.match(findSerialNumberRegex);

      probeState = {
        state: State.Known,
        data: {
          identifier: identifierMatches![0],
          serialNumber:
            serialNumberMatches![1] == "unknown"
              ? null
              : serialNumberMatches![1],
        },
      };
      break;
    }
  }

  submitProbe({ probePos: props.channel, probeState: probeState });
}
</script>

<template>
  <v-card elevation="1">
    <v-card-title prepend-icon="mdi-chip">
      <v-icon icon="mdi-ray-start-end" size="40" class="mr-2" />
      Testchannel {{ props.channel }}
    </v-card-title>

    <v-card-text class="pb-0" style="margin-bottom: 1vh">
      <v-autocomplete @update:modelValue="submit" v-model="selectedProbe" :loading="loading" :items="availableProbes"
        dense label="Debug Probe" hint="Please select the corresponding probe" persistent-hint
        no-data-text="No probes found">
      </v-autocomplete>
    </v-card-text>
  </v-card>
</template>
