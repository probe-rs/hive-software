<script setup lang="ts">
import {
  type BackendMutation,
  type BackendQuery,
  ResultEnum,
} from "@/gql/backend";

import TargetOverview from "@/components/TargetOverview.vue";
import { defineProps } from "vue";
import { useAppConfig } from "@/stores/appConfig";
import { computed, ref } from "@vue/reactivity";
import { AppTheme } from "@/plugins/vuetify";
import SuccessSnackbar from "@/components/SuccessSnackbar.vue";
import { useMutation, useQuery } from "@vue/apollo-composable";
import gql from "graphql-tag";

// Assets
import ferrisGesture from "@/assets/ferris/rustacean-flat-gesture.svg";

const appConfig = useAppConfig();

const props = defineProps({
  tssPos: { type: Number, required: true },
});

const { loading, result, refetch } = useQuery<BackendQuery>(gql`
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
`);

const reloadSuccess = ref(false);
const {
  mutate: reloadTestrack,
  loading: testrackLoading,
  onDone: onReloadDone,
} = useMutation<BackendMutation>(
  gql`
    mutation {
      reinitializeHardware
    }
  `,
  { fetchPolicy: "no-cache" },
);

onReloadDone(() => {
  reloadSuccess.value = true;

  // refetch query as some data might have changed after hardware reinitialization
  refetch();
});

const assignedTargets = computed(() => {
  if (result.value) {
    return result.value.assignedTargets;
  }
  return [];
});

const tssPos = computed(() => {
  return props.tssPos;
});

const hasDaughterboard = computed(() => {
  if (assignedTargets.value[tssPos.value]) {
    return true;
  }
  return false;
});
</script>

<template>
  <v-row>
    <v-col cols="12">
      <v-sheet rounded class="pa-4" color="transparent">
        <v-row class="pa-2">
          <h2 class="align-self-center">Target Stack Shield {{ tssPos }}</h2>

          <v-spacer />

          <v-icon size="25" class="align-self-center" :icon="hasDaughterboard ? 'mdi-card' : 'mdi-card-remove'"
            :color="hasDaughterboard ? 'success' : 'info'" />
          <p class="align-self-center pl-2">
            {{
                hasDaughterboard
                  ? "Daughterboard Connected"
                  : "No Daughterboard Found"
            }}
          </p>
        </v-row>
      </v-sheet>
      <v-divider />
    </v-col>
  </v-row>

  <template v-if="hasDaughterboard && !loading">
    <v-row>
      <v-col sm="6">
        <TargetOverview :tssPos="tssPos" :target="0"
          :status="assignedTargets[tssPos]![0].data ? assignedTargets[tssPos]![0].data!.flashStatus : ResultEnum.Error"
          :statusMessage="assignedTargets[tssPos]![0].data ? assignedTargets[tssPos]![0].data!.flashMessage : ''"
          :initialData="assignedTargets[tssPos]![0]" />
      </v-col>
      <v-col sm="6">
        <TargetOverview :tssPos="tssPos" :target="1"
          :status="assignedTargets[tssPos]![1].data ? assignedTargets[tssPos]![1].data!.flashStatus : ResultEnum.Error"
          :statusMessage="assignedTargets[tssPos]![1].data ? assignedTargets[tssPos]![1].data!.flashMessage : ''"
          :initialData="assignedTargets[tssPos]![1]" />
      </v-col>
    </v-row>
    <v-row>
      <v-col sm="6">
        <TargetOverview :tssPos="tssPos" :target="2"
          :status="assignedTargets[tssPos]![2].data ? assignedTargets[tssPos]![2].data!.flashStatus : ResultEnum.Error"
          :statusMessage="assignedTargets[tssPos]![2].data ? assignedTargets[tssPos]![2].data!.flashMessage : ''"
          :initialData="assignedTargets[tssPos]![2]" />
      </v-col>
      <v-col sm="6">
        <TargetOverview :tssPos="tssPos" :target="3"
          :status="assignedTargets[tssPos]![3].data ? assignedTargets[tssPos]![3].data!.flashStatus : ResultEnum.Error"
          :statusMessage="assignedTargets[tssPos]![3].data ? assignedTargets[tssPos]![3].data!.flashMessage : ''"
          :initialData="assignedTargets[tssPos]![3]" />
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12">
        <v-sheet rounded elevation="1" class="pa-4">
          <v-row class="pa-2">
            <v-spacer />
            <v-btn v-if="!testrackLoading" color="success" variant="text" @click="reloadTestrack">
              Reload Testrack
            </v-btn>
            <v-progress-linear v-else indeterminate color="secondary" />
          </v-row>
        </v-sheet>
      </v-col>
    </v-row>

    <SuccessSnackbar :isSuccess="reloadSuccess" message="Testrack successfully reloaded"
      @closeEvent="reloadSuccess = false" />
  </template>

  <template v-else-if="!hasDaughterboard && !loading">
    <v-row>
      <v-col cols="12">
        <v-sheet rounded elevation="1" class="pa-4">
          <v-row class="pa-6">
            <v-img :src="ferrisGesture" height="125" :style="
              appConfig.theme == AppTheme.Light
                ? ''
                : 'filter: brightness(80%);'
            " />
          </v-row>
          <v-row class="pa-2 justify-center">
            <p class="align-self-center" style="
                max-width: 70%;
                text-align: center;
                color: rgb(
                  var(--v-theme-on-surface),
                  var(--v-disabled-opacity)
                );
              ">
              Could not detect any Daughterboard on this Target Stack Shield. If
              a Daughterboard is connected but not shown in here it might be
              related to a hardware problem. In that case, please make sure to
              check if the Daughterboard detect pin on the Daughterboard
              correctly outputs 3.3V to the IO-Expander.
            </p>
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
</template>
