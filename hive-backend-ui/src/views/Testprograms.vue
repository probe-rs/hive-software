<script setup lang="ts">
import { ref } from "vue";
import { computed } from "@vue/reactivity";
import { useMutation, useQuery } from "@vue/apollo-composable";
import {
  Architecture,
  type BackendMutation,
  type BackendMutationCreateTestprogramArgs,
  type BackendMutationDeleteTestprogramArgs,
  type BackendMutationSetActiveTestprogramArgs,
  type BackendQuery,
} from "@/gql/backend";
import gql from "graphql-tag";
import Testprogram from "@/components/Testprogram.vue";
import ErrorSnackbar from "@/components/ErrorSnackbar.vue";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import { cloneDeep } from "@apollo/client/utilities";

const DEFAULT_TESTPROGRAM = "default";

const selectedTestprogram = ref(DEFAULT_TESTPROGRAM);
const selectedArchitecture = ref(Architecture.Arm);
const createTestprogramDialog = ref(false);
const isError = ref(false);
const errorMessage = ref("");

const { result } = useQuery<BackendQuery>(
  gql`
    query {
      availableTestprograms {
        name
      }
      activeTestprogram
    }
  `,
);

const testprograms = computed(() => {
  if (result.value) {
    let testprograms: Array<string> = [];
    result.value.availableTestprograms.forEach((e) =>
      testprograms.push(e.name),
    );

    return testprograms;
  }
  return [DEFAULT_TESTPROGRAM];
});

const activeTestprogram = computed(() => {
  if (result.value) {
    return result.value.activeTestprogram;
  }
  return DEFAULT_TESTPROGRAM;
});

const selectedIsActive = computed(() => {
  return activeTestprogram.value === selectedTestprogram.value;
});

const { mutate: setActiveTestprogram } = useMutation<
  BackendMutation,
  BackendMutationSetActiveTestprogramArgs
>(
  gql`
    mutation ($testprogramName: String!) {
      setActiveTestprogram(testprogramName: $testprogramName)
    }
  `,
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const setActiveTestprogram = data.setActiveTestprogram;

      const QUERY = gql`
        query {
          activeTestprogram
        }
      `;

      let cacheData: BackendQuery | null = cache.readQuery({
        query: QUERY,
      });

      if (!cacheData) {
        return;
      }

      cacheData = {
        ...cacheData,
        activeTestprogram: setActiveTestprogram,
      };

      cache.writeQuery<BackendQuery>({ query: QUERY, data: cacheData });
    },
  },
);

const {
  mutate: createTestprogram,
  onDone: onTestprogramCreated,
  onError: OnTestprogramCreateError,
  loading: testprogramCreateLoading,
} = useMutation<BackendMutation, BackendMutationCreateTestprogramArgs>(
  gql`
    mutation ($testprogramName: String!) {
      createTestprogram(testprogramName: $testprogramName) {
        name
        testprogramArm {
          architecture
          status
          compileMessage
        }
        testprogramRiscv {
          architecture
          status
          compileMessage
        }
      }
    }
  `,
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const createTestprogram = data.createTestprogram;

      const QUERY = gql`
        query {
          availableTestprograms {
            name
            testprogramArm {
              architecture
              status
              compileMessage
            }
            testprogramRiscv {
              architecture
              status
              compileMessage
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

      const newAvailableTestprograms = cloneDeep(
        cacheData.availableTestprograms,
      );
      newAvailableTestprograms.push(createTestprogram);

      cacheData = {
        ...cacheData,
        availableTestprograms: newAvailableTestprograms,
      };

      cache.writeQuery<BackendQuery>({ query: QUERY, data: cacheData });
    },
  },
);

const newTestprogramName = ref("");

onTestprogramCreated((response) => {
  closeCreateTestprogramDialog();

  if (response.data) {
    selectedTestprogram.value = response.data.createTestprogram.name;
  }
});

OnTestprogramCreateError((response) => {
  errorMessage.value = response.message;
  isError.value = true;
});

function closeCreateTestprogramDialog() {
  createTestprogramDialog.value = false;
  newTestprogramName.value = "";
}

const { mutate: deleteTestprogram, onDone: onTestprogramDeleted } = useMutation<
  BackendMutation,
  BackendMutationDeleteTestprogramArgs
>(
  gql`
    mutation ($testprogramName: String!) {
      deleteTestprogram(testprogramName: $testprogramName)
    }
  `,
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const deleteTestprogram = data.deleteTestprogram;

      const QUERY = gql`
        query {
          availableTestprograms {
            name
            testprogramArm {
              architecture
              status
              compileMessage
            }
            testprogramRiscv {
              architecture
              status
              compileMessage
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

      const newAvailableTestprograms = cloneDeep(
        cacheData.availableTestprograms,
      );
      const idx = newAvailableTestprograms.findIndex(
        (e) => e.name === deleteTestprogram,
      );

      newAvailableTestprograms.splice(idx, 1);

      cacheData = {
        ...cacheData,
        availableTestprograms: newAvailableTestprograms,
      };

      cache.writeQuery<BackendQuery>({ query: QUERY, data: cacheData });
    },
  },
);

const showDeleteTestprogramConfirmDialog = ref(false);

function deleteCurrentTestprogram() {
  showDeleteTestprogramConfirmDialog.value = false;

  deleteTestprogram({ testprogramName: selectedTestprogram.value });
}

onTestprogramDeleted(() => {
  selectedTestprogram.value = DEFAULT_TESTPROGRAM;
});
</script>

<template>
  <v-toolbar color="surface" elevation="1">
    <v-icon size="25" class="ml-2" icon="mdi-file-code-outline" />

    <v-toolbar-title>Testprogram</v-toolbar-title>

    <template v-slot:extension>
      <v-tabs color="secondary" v-model="selectedArchitecture">
        <v-tab :value="Architecture.Arm"> ARM </v-tab>
        <v-tab :value="Architecture.Riscv"> RISCV </v-tab>
      </v-tabs>

      <v-spacer></v-spacer>
      <v-btn :color="selectedIsActive ? 'disabled' : 'info'" :disabled="selectedIsActive"
        @click="setActiveTestprogram({ testprogramName: selectedTestprogram })">Set active</v-btn>
      <v-btn v-if="selectedTestprogram !== DEFAULT_TESTPROGRAM" color="error"
        @click="showDeleteTestprogramConfirmDialog = true">Delete Testprogram</v-btn>
      <v-btn v-if="selectedTestprogram !== DEFAULT_TESTPROGRAM" color="success">Check & Save</v-btn>
    </template>

    <v-spacer />

    <v-icon v-if="selectedIsActive" color="success" icon="mdi-check-circle-outline" />
    <!--<v-tooltip activator="parent" location="bottom end" origin="top start">Testprogram is currently active</v-tooltip>-->

    <v-btn>
      {{ selectedTestprogram }}
      <v-menu activator="parent">
        <v-list>
          <v-list-item v-for="testprogram in testprograms" :key="testprogram" :value="testprogram"
            @click="selectedTestprogram = testprogram">
            <v-list-item-title>{{
                testprogram.toUpperCase()
            }}</v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>
    </v-btn>

    <v-btn icon @click="createTestprogramDialog = true">
      <v-icon>mdi-plus</v-icon>
      <v-tooltip activator="parent" location="bottom end">Add testprogram</v-tooltip>
    </v-btn>
  </v-toolbar>

  <v-dialog v-model="createTestprogramDialog" persistent max-width="800px" transition="dialog-top-transition">
    <v-card style="min-width: 50vw">
      <v-card-title class="text-h5 grey lighten-2">
        Create new testprogram
      </v-card-title>

      <v-card-text>
        <v-form>
          <v-text-field v-model="newTestprogramName" label="Testprogram name" variant="underlined" density="compact" />
        </v-form>
      </v-card-text>

      <v-divider></v-divider>

      <v-card-actions>
        <v-btn color="error" text @click="closeCreateTestprogramDialog">
          Cancel
        </v-btn>
        <v-spacer></v-spacer>
        <v-btn color="success" text @click="createTestprogram({ testprogramName: newTestprogramName })">
          Create new testprogram
        </v-btn>
      </v-card-actions>

      <!--Replace with loading save and exit button once available in vuetify-->
      <v-overlay v-model="testprogramCreateLoading" contained class="align-center justify-center">
        <v-progress-circular size="80" color="secondary" indeterminate />
      </v-overlay>
    </v-card>
  </v-dialog>

  <Testprogram :testprogram-name="selectedTestprogram" :selected-architecture="selectedArchitecture" />

  <ConfirmDialog :is-active="showDeleteTestprogramConfirmDialog" @cancel="showDeleteTestprogramConfirmDialog = false"
    @confirm="deleteCurrentTestprogram"
    :text="`Do you really want to delete the testprogram '${selectedTestprogram}'?`" />

  <ErrorSnackbar :is-error="isError" :message="errorMessage" @close-event="isError = false" />
</template>
