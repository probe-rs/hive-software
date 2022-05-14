<script setup lang="ts">
import ProbeOverview from "@/components/ProbeOverview.vue";
import { useQuery } from "@vue/apollo-composable";
import gql from "graphql-tag";
import { computed } from "vue";

type AssignedProbeData = {
  state: string;
  data: { identifier: string; serialNumber: string | null } | null;
};

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

const hasUnassignedProbes = computed(() => {
  return (
    connectedProbes.value.length >
    assignedProbes.value.filter((assignedProbe: AssignedProbeData) => {
      if (assignedProbe.state === "KNOWN") {
        return true;
      }
      return false;
    }).length
  );
});
</script>

<template>
  <v-row>
    <v-col cols="12">
      <v-sheet rounded class="pa-4" color="transparent">
        <v-row class="pa-2">
          <h2 class="align-self-center">Probe Stack Shield</h2>

          <v-spacer />

          <v-icon
            size="25"
            class="align-self-center"
            :icon="
              !hasUnassignedProbes ? 'mdi-checkbox-marked' : 'mdi-help-box'
            "
            :color="!hasUnassignedProbes ? 'success' : 'info'"
          />
          <p class="align-self-center pl-2">
            {{
              !hasUnassignedProbes
                ? "All probes are assigned to a channel"
                : "Detected unassigned probes"
            }}
          </p>
        </v-row>
      </v-sheet>
      <v-divider />
    </v-col>
  </v-row>

  <template v-if="!loading">
    <v-row>
      <v-col sm="6">
        <ProbeOverview :channel="0" :initialData="assignedProbes[0]" />
      </v-col>
      <v-col sm="6">
        <ProbeOverview :channel="1" :initialData="assignedProbes[1]" />
      </v-col>
    </v-row>
    <v-row>
      <v-col sm="6">
        <ProbeOverview :channel="2" :initialData="assignedProbes[2]" />
      </v-col>
      <v-col sm="6">
        <ProbeOverview :channel="3" :initialData="assignedProbes[3]" />
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12">
        <v-sheet rounded elevation="1" class="pa-4">
          <v-row class="pa-2">
            <v-spacer />
            <v-btn color="success" variant="text"> Reload Testrack </v-btn>
          </v-row>
        </v-sheet>
      </v-col>
    </v-row>
  </template>

  <template v-else>
    <!--TODO: Replace with skeleton loader once it is supported by vuetify-->
    <v-row>
      <v-col cols="12">
        <v-row class="pa-6 justify-center">
          <v-progress-linear indeterminate color="secondary" />
        </v-row>
        <v-row class="justify-center">
          <p
            class="align-self-center"
            style="
              max-width: 70%;
              text-align: center;
              color: rgb(var(--v-theme-on-surface), var(--v-disabled-opacity));
            "
          >
            Loading data...
          </p>
        </v-row>
      </v-col>
    </v-row>
  </template>
</template>
