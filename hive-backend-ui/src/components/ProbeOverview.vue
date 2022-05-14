<script setup lang="ts">
import { cloneDeep } from "@apollo/client/utilities";
import { useMutation, useQuery } from "@vue/apollo-composable";
import gql from "graphql-tag";
import { computed, ref, type PropType } from "vue";

type ProbeData = { identifier: string; serialNumber: string | null };
type AssignedProbeData = {
  state: string;
  data: { identifier: string; serialNumber: string | null } | null;
};

const props = defineProps({
  channel: {
    type: Number,
    required: true,
  },
  initialData: {
    type: Object as PropType<AssignedProbeData>,
    required: true,
  },
});

const selectedProbe = ref(displayAssignedProbe(props.initialData));

const { loading, result } = useQuery(gql`
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

const { mutate: submitProbe } = useMutation(
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
    update: (cache, { data: { assignProbe } }) => {
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
      let data: any = cache.readQuery({
        query: QUERY,
      });

      const newAssignedProbes = cloneDeep(data.assignedProbes);

      newAssignedProbes[assignProbe.probePos] = assignProbe.data;

      data = {
        ...data,
        assignedProbes: newAssignedProbes,
      };

      cache.writeQuery({ query: QUERY, data });
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
    (connectedProbe: ProbeData) => {
      for (let i = 0; i < assignedProbes.value.length; i++) {
        if (assignedProbes.value[i].state !== "KNOWN") {
          continue;
        }

        if (
          assignedProbes.value[i].data.identifier ===
            connectedProbe.identifier &&
          assignedProbes.value[i].data.serialNumber ===
            connectedProbe.serialNumber
        ) {
          return false;
        }
      }
      return true;
    },
  );

  const displayAvailable = available.map((availableProbe: ProbeData) => {
    return displayProbe(availableProbe);
  });

  displayAvailable.push("Unknown", "Not Connected");

  return displayAvailable;
});

function displayAssignedProbe(probe: AssignedProbeData): string {
  switch (probe.state) {
    case "KNOWN":
      return `${probe.data!.identifier} (S/N: ${
        probe.data!.serialNumber ? probe.data!.serialNumber : "unknown"
      })`;
    case "UNKNOWN":
      return "Unknown";
    case "NOT_CONNECTED":
      return "Not Connected";
    default:
      return `Received unknown state: ${probe.state}`;
  }
}

function displayProbe(probe: ProbeData): string {
  return `${probe.identifier} (S/N: ${
    probe.serialNumber ? probe.serialNumber : "unknown"
  })`;
}

function submit(probeName: string) {
  console.log("submitting probe: " + probeName);
  let probeState;

  switch (probeName) {
    case "Unknown":
      probeState = {
        state: "UNKNOWN",
        data: null,
      };
      break;
    case "Not Connected":
      probeState = {
        state: "NOT_CONNECTED",
        data: null,
      };
      break;
    default:
      const findIdentifierRegex = /.+?(?= \(S\/N:)/;
      const findSerialNumberRegex = /\(S\/N: (.*?)\)/;

      const identifierMatches = probeName.match(findIdentifierRegex);
      const serialNumberMatches = probeName.match(findSerialNumberRegex);

      probeState = {
        state: "KNOWN",
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

  submitProbe({ probePos: props.channel, probeState: probeState });
}
</script>

<template>
  <v-card elevation="1">
    <v-card-title prepend-icon="mdi-chip">
      <v-icon icon="mdi-ray-start-end" size="40" class="mr-2" />
      Testchannel {{ props.channel }}
    </v-card-title>

    <v-card-text class="pb-0">
      <v-autocomplete
        @update:modelValue="submit"
        v-model="selectedProbe"
        :loading="loading"
        :items="availableProbes"
        dense
        label="Debug Probe"
        hint="Please select the corresponding probe"
        persistent-hint
        no-data-text="No probes found"
      >
      </v-autocomplete>
    </v-card-text>
  </v-card>
</template>
