<script setup lang="ts">
import type {
    BackendQuery,
} from "@/gql/backend";

import ProbeOverview from "@/components/ProbeOverview.vue";
import { useMutation, useQuery } from "@vue/apollo-composable";
import gql from "graphql-tag";
import { computed, ref, watch, type ComputedRef } from "vue";
import ErrorSnackbar from "./ErrorSnackbar.vue";

const { loading, result, onError } = useQuery<BackendQuery>(gql`
  query {
    systemInfo {
      controller
      soc
      hostname
      cores
      os
      memory {
        total
        free
      }
      disk {
        total
        free
      }
      averageLoad
    }
  }
`, null,
    {
        fetchPolicy: "network-only",
        pollInterval: 8000,
    });

const systemInfo = computed(() => {
    if (!result.value) {
        return null;
    }

    const systemInfo = result.value.systemInfo;

    return {
        memoryTotal: Math.round((systemInfo.memory.total / 1000000) * 1000) / 1000,
        memoryFree: Math.round((systemInfo.memory.free / 1000000) * 1000) / 1000,
        diskTotal: Math.round((systemInfo.disk.total / 1000000) * 1000) / 1000,
        diskFree: Math.round((systemInfo.disk.free / 1000000) * 1000) / 1000,
        averageLoad: systemInfo.averageLoad,
        controller: systemInfo.controller,
        cores: systemInfo.cores,
        soc: systemInfo.soc,
        hostname: systemInfo.hostname,
        os: systemInfo.os,
    }
});

const memoryUsage = computed(() => {
    if (!systemInfo.value || systemInfo.value.memoryTotal === 0) {
        return 0;
    }

    return ((systemInfo.value.memoryTotal - systemInfo.value.memoryFree) / systemInfo.value.memoryTotal) * 100;
});

const diskUsage = computed(() => {
    if (!systemInfo.value || systemInfo.value.diskTotal === 0) {
        return 0;
    }

    return ((systemInfo.value.diskTotal - systemInfo.value.diskFree) / systemInfo.value.diskTotal) * 100;
});

const averageLoad = computed(() => {
    if (!systemInfo.value || systemInfo.value.cores === 0) {
        return 0;
    }

    return (systemInfo.value.averageLoad / systemInfo.value.cores) * 100;
})

const errorMessage = ref("");
const showError = ref(false);

onError((error) => {
    errorMessage.value = error.message;
    showError.value = true;
})

</script>

<template>
    <v-row>
        <v-col cols="12">
            <v-sheet rounded class="pa-4" color="transparent">
                <v-row class="pa-2">
                    <h2 class="align-self-center">Controller</h2>

                    <v-spacer />
                </v-row>
            </v-sheet>
            <v-divider />
        </v-col>
    </v-row>

    <template v-if="!loading">
        <v-row>
            <v-col cols="12">
                <v-sheet rounded elevation="1" class="pa-4">
                    <v-row class="d-flex pa-2">
                        <div style="min-width: 8vw">
                            <strong>Controller:</strong>
                        </div>
                        <div style="flex-grow: 1;" class="pl-2">
                            <p>{{ systemInfo!.controller }}</p>
                        </div>
                    </v-row>
                    <v-row class="d-flex pa-2">
                        <div style="min-width: 8vw">
                            <strong>SOC:</strong>
                        </div>
                        <div style="flex-grow: 1;" class="pl-2">
                            <p>{{ systemInfo!.soc }}</p>
                        </div>
                    </v-row>
                    <v-row class="d-flex pa-2">
                        <div style="min-width: 8vw">
                            <strong>Hostname:</strong>
                        </div>
                        <div style="flex-grow: 1;" class="pl-2">
                            <p>{{ systemInfo!.hostname }}</p>
                        </div>
                    </v-row>
                    <v-row class="d-flex pa-2">
                        <div style="min-width: 8vw">
                            <strong>OS:</strong>
                        </div>
                        <div style="flex-grow: 1;" class="pl-2">
                            <p>{{ systemInfo!.os }}</p>
                        </div>
                    </v-row>
                    <v-row class="d-flex pa-2">
                        <div style="min-width: 8vw">
                            <strong>Memory:</strong>
                        </div>
                        <div style="flex-grow: 1;" class="pl-2">
                            <v-progress-linear v-model="memoryUsage" rounded color="secondary" height="25">
                                <template v-slot:default>
                                    <strong> {{ Math.round((systemInfo!.memoryTotal - systemInfo!.memoryFree) * 1000) /
                                            1000
                                    }} / {{ systemInfo!.memoryTotal }} GB</strong>
                                </template>
                            </v-progress-linear>
                        </div>
                    </v-row>
                    <v-row class="d-flex pa-2">
                        <div style="min-width: 8vw">
                            <strong>Disk Usage:</strong>
                        </div>
                        <div style="flex-grow: 1;" class="pl-2">
                            <v-progress-linear v-model="diskUsage" rounded color="secondary" height="25">
                                <template v-slot:default>
                                    <strong>{{ Math.round((systemInfo!.diskTotal - systemInfo!.diskFree) * 1000) / 1000
                                    }} / {{ systemInfo!.diskTotal }}
                                        GB</strong>
                                </template>
                            </v-progress-linear>
                        </div>
                    </v-row>
                    <v-row class="d-flex pa-2">
                        <div style="min-width: 8vw">
                            <strong>Avg. System Load:</strong>
                        </div>
                        <div style="flex-grow: 1;" class="pl-2">
                            <v-progress-linear v-model="averageLoad" rounded color="secondary" height="25">
                                <template v-slot:default="{ value }">
                                    <strong>{{ Math.ceil(value) }}%</strong>
                                </template>
                            </v-progress-linear>
                        </div>
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
                    <p class="align-self-center" style="
              max-width: 70%;
              text-align: center;
              color: rgb(var(--v-theme-on-surface), var(--v-disabled-opacity));
            ">
                        Loading data...
                    </p>
                </v-row>
            </v-col>
        </v-row>
    </template>

    <ErrorSnackbar :is-error="showError" :message="errorMessage" @close-event="showError = false" />
</template>
