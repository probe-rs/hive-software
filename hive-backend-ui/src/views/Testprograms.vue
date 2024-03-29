<script setup lang="ts">
import { ref } from "vue";
import { computed } from "vue";
import { useMutation, useQuery } from "@vue/apollo-composable";
import { gql } from "@/gql-schema";
import { Architecture } from "@/gql-schema/graphql";
import Testprogram from "@/components/Testprogram.vue";
import ErrorSnackbar from "@/components/ErrorSnackbar.vue";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import { cloneDeep } from "@apollo/client/utilities";

const DEFAULT_TESTPROGRAM = "default";

const selectedTestprogram = ref(DEFAULT_TESTPROGRAM);
const selectedArchitecture = ref(Architecture.Arm);
const createTestprogramDialog = ref(false);
const deleteSelectedTestprogram = ref(false);
const isError = ref(false);
const errorMessage = ref("");
// Flag which is true if there have been unsaved changed
const unsavedChanges = ref(false);

const { result, refetch: refetchTestprograms } = useQuery(
  gql(`
  query AvailableAndActiveTestPrograms {
    availableTestprograms {
      name
    }
    activeTestprogram
  }
`),
);

const testprograms = computed(() => {
  if (result.value) {
    const testprograms: Array<string> = [];
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

const { mutate: setActiveTestprogram } = useMutation(
  gql(`
    mutation SetActiveTestProgram ($testprogramName: String!) {
      setActiveTestprogram(testprogramName: $testprogramName)
    }
  `),
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const setActiveTestprogram = data.setActiveTestprogram;

      const QUERY = gql(`
        query ActiveTestProgram {
          activeTestprogram
        }
      `);

      let cacheData = cache.readQuery({
        query: QUERY,
      });

      if (!cacheData) {
        return;
      }

      cacheData = {
        ...cacheData,
        activeTestprogram: setActiveTestprogram,
      };

      cache.writeQuery({ query: QUERY, data: cacheData });
    },
  },
);

const {
  mutate: createTestprogram,
  onDone: onTestprogramCreated,
  onError: OnTestprogramCreateError,
  loading: testprogramCreateLoading,
} = useMutation(
  gql(`
    mutation CreateTestProgram ($testprogramName: String!) {
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
  `),
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const createTestprogram = data.createTestprogram;

      const QUERY = gql(`
        query AvailableTestprogramsOverview {
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
      `);

      let cacheData = cache.readQuery({
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

      cache.writeQuery({ query: QUERY, data: cacheData });
    },
  },
);

const newTestprogramName = ref("");

onTestprogramCreated((response) => {
  closeCreateTestprogramDialog();

  if (response.data) {
    selectedTestprogram.value = response.data.createTestprogram.name;
    refetchTestprograms();
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

function onTestprogramDeleted() {
  selectedTestprogram.value = DEFAULT_TESTPROGRAM;
  deleteSelectedTestprogram.value = false;
  refetchTestprograms();
}

const showConfirmChangeSelectedProgram = ref(false);
const selectedTestprogramCandidate = ref("");

function changeSelectedTestprogram(testprogram: string) {
  if (unsavedChanges.value) {
    showConfirmChangeSelectedProgram.value = true;
    selectedTestprogramCandidate.value = testprogram;
    return;
  }

  selectedTestprogram.value = testprogram;
}

function confirmChangeSelectedProgram() {
  showConfirmChangeSelectedProgram.value = false;
  selectedTestprogram.value = selectedTestprogramCandidate.value;
}
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
      <v-btn
        :color="selectedIsActive ? 'disabled' : 'info'"
        :disabled="selectedIsActive"
        @click="setActiveTestprogram({ testprogramName: selectedTestprogram })"
        >Set active</v-btn
      >
      <v-btn
        v-if="selectedTestprogram !== DEFAULT_TESTPROGRAM"
        color="error"
        @click="deleteSelectedTestprogram = true"
      >
        Delete Testprogram</v-btn
      >
      <v-btn v-if="selectedTestprogram !== DEFAULT_TESTPROGRAM" color="success"
        >Check & Save</v-btn
      >
    </template>

    <v-spacer />

    <v-icon
      v-if="selectedIsActive"
      color="success"
      icon="mdi-check-circle-outline"
    />
    <!--<v-tooltip activator="parent" location="bottom end" origin="top start">Testprogram is currently active</v-tooltip>-->

    <v-btn>
      {{ selectedTestprogram }}
      <v-menu activator="parent">
        <v-list>
          <v-list-item
            v-for="testprogram in testprograms"
            :key="testprogram"
            :value="testprogram"
            @click="changeSelectedTestprogram(testprogram)"
            :v-if="testprogram !== selectedTestprogram"
          >
            <v-list-item-title>{{
              testprogram.toUpperCase()
            }}</v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>
    </v-btn>

    <v-btn icon @click="createTestprogramDialog = true">
      <v-icon>mdi-plus</v-icon>
      <v-tooltip activator="parent" location="bottom end"
        >Add testprogram</v-tooltip
      >
    </v-btn>
  </v-toolbar>

  <v-dialog
    v-model="createTestprogramDialog"
    persistent
    max-width="800px"
    transition="dialog-top-transition"
  >
    <v-card style="min-width: 50vw">
      <v-card-title class="text-h5 grey lighten-2">
        Create new testprogram
      </v-card-title>

      <v-card-text>
        <v-form>
          <v-text-field
            v-model="newTestprogramName"
            label="Testprogram name"
            variant="underlined"
            density="compact"
          />
        </v-form>
      </v-card-text>

      <v-divider></v-divider>

      <v-card-actions>
        <v-btn
          color="error"
          variant="text"
          @click="closeCreateTestprogramDialog"
        >
          Cancel
        </v-btn>
        <v-spacer></v-spacer>
        <v-btn
          color="success"
          variant="text"
          @click="createTestprogram({ testprogramName: newTestprogramName })"
        >
          Create new testprogram
        </v-btn>
      </v-card-actions>

      <!--Replace with loading save and exit button once available in vuetify-->
      <v-overlay
        v-model="testprogramCreateLoading"
        contained
        class="align-center justify-center"
      >
        <v-progress-circular size="80" color="secondary" indeterminate />
      </v-overlay>
    </v-card>
  </v-dialog>

  <Testprogram
    :deleteTestprogramEvent="deleteSelectedTestprogram"
    :testprogram-name="selectedTestprogram"
    :selected-architecture="selectedArchitecture"
    @testprogramDeleted="onTestprogramDeleted"
    @testprogramNotDeleted="deleteSelectedTestprogram = false"
    @code-edited="unsavedChanges = true"
  />

  <ConfirmDialog
    :is-active="showConfirmChangeSelectedProgram"
    text="Current Testprogram has unsaved changes which will get lost if you change testprograms. Proceed anyway?"
    @confirm="confirmChangeSelectedProgram"
    @cancel="showConfirmChangeSelectedProgram = false"
  />

  <ErrorSnackbar
    :is-error="isError"
    :message="errorMessage"
    @close-event="isError = false"
  />
</template>
