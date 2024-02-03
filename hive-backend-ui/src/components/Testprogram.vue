<script setup lang="ts">
import CodeEditor from "@/components/CodeEditor.vue";
import Terminal from "@/components/Terminal.vue";
import {
  Architecture,
  type BackendMutation,
  type BackendMutationDeleteTestprogramArgs,
  type BackendMutationModifyTestprogramArgs,
  type BackendQuery,
  type BackendQueryTestprogramArgs,
  type FullTestProgramResponse,
} from "@/gql/backend";
import { useMutation, useQuery } from "@vue/apollo-composable";
import { computed, ref, toRefs } from "vue";
import gql from "graphql-tag";
import { watch, type PropType } from "vue";
import * as base64 from "base-64";
import { cloneDeep } from "@apollo/client/utilities";
import ConfirmDialog from "./ConfirmDialog.vue";
import type { Maybe } from "@/gql/baseTypes";

const DEFAULT_TESTPROGRAM = "default";

const props = defineProps({
  testprogramName: {
    type: String,
    required: true,
  },
  selectedArchitecture: {
    type: String as PropType<Architecture>,
    required: true,
  },
  deleteTestprogramEvent: {
    type: Boolean,
    required: true,
  },
});

const { selectedArchitecture, testprogramName, deleteTestprogramEvent } =
  toRefs(props);

const emit = defineEmits([
  "testprogramDeleted",
  "testprogramNotDeleted",
  "codeEdited",
]);

const testprogram = ref<Maybe<FullTestProgramResponse>>(null);
const codeArm = ref("");
const codeRiscv = ref("");

const {
  loading,
  onResult: onTestprogramResult,
  refetch: refetchTestprogram,
} = useQuery<BackendQuery, BackendQueryTestprogramArgs>(
  gql`
    query ($testprogramName: String!) {
      testprogram(testprogramName: $testprogramName) {
        testprogram {
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
        codeArm
        codeRiscv
      }
    }
  `,
  {
    testprogramName: testprogramName.value,
  },
);

watch(testprogramName, (newName) => {
  // If selected testprogram changed we fetch the new testprogram data
  refetchTestprogram({ testprogramName: newName });
});

onTestprogramResult((result) => {
  testprogram.value = result.data.testprogram;

  codeArm.value = base64.decode(result.data.testprogram.codeArm);
  codeRiscv.value = base64.decode(result.data.testprogram.codeRiscv);
});

const code = computed(() => {
  switch (selectedArchitecture.value) {
    case Architecture.Arm:
      return codeArm;
    case Architecture.Riscv:
      return codeRiscv;
  }
});

const compileMessage = computed(() => {
  if (!testprogram.value) {
    return "loading output...";
  }

  switch (selectedArchitecture.value) {
    case Architecture.Arm:
      return testprogram.value.testprogram.testprogramArm.compileMessage;
    case Architecture.Riscv:
      return testprogram.value.testprogram.testprogramRiscv.compileMessage;
  }
});

watch(deleteTestprogramEvent, (isDelete) => {
  if (isDelete) {
    showDeleteTestprogramConfirmDialog.value = true;
  }
});

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

  deleteTestprogram({ testprogramName: testprogramName.value });
}

function cancelTestprogramDelete() {
  showDeleteTestprogramConfirmDialog.value = false;
  emit("testprogramNotDeleted");
}

onTestprogramDeleted(() => {
  emit("testprogramDeleted");
});

const {
  mutate: modifyTestprogram,
  onDone: onTestprogramModified,
  onError: OnTestprogramModifyError,
  loading: testprogramModifyLoading,
} = useMutation<BackendMutation, BackendMutationModifyTestprogramArgs>(
  gql`
    mutation ($testprogramName: String!, $codeFiles: FileList!) {
      modifyTestprogram(
        testprogramName: $testprogramName
        codeFiles: $codeFiles
      ) {
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

      const modifyTestprogram = data.modifyTestprogram;

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
      const modifyIndex = newAvailableTestprograms.findIndex((element) => {
        element.name === modifyTestprogram.name;
      });

      newAvailableTestprograms[modifyIndex] = modifyTestprogram;

      cacheData = {
        ...cacheData,
        availableTestprograms: newAvailableTestprograms,
      };

      cache.writeQuery<BackendQuery>({ query: QUERY, data: cacheData });
    },
  },
);

async function fileUpload(file: Array<File>) {
  code.value.value = await file[0].text();
}

function codeChange(newCode: string) {
  switch (selectedArchitecture.value) {
    case Architecture.Arm:
      if (newCode !== codeArm.value) {
        emit("codeEdited");
        codeArm.value = newCode;
      }
      break;
    case Architecture.Riscv:
      if (newCode !== codeArm.value) {
        codeRiscv.value = newCode;
        emit("codeEdited");
      }
      break;
  }
}
</script>

<template>
  <v-row class="pt-4">
    <v-col style="height: 55vh">
      <CodeEditor
        v-if="!loading"
        :code="code.value"
        @code-change="codeChange"
        :read-only="testprogramName === DEFAULT_TESTPROGRAM"
      />
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
                  color: rgb(
                    var(--v-theme-on-surface),
                    var(--v-disabled-opacity)
                  );
                "
              >
                Loading data...
              </p>
            </v-row>
          </v-col>
        </v-row>
      </template>
    </v-col>
  </v-row>

  <v-row>
    <v-col style="height: 30vh" lg="8">
      <Terminal
        v-if="!loading"
        :content="compileMessage"
        :scrollToBottom="false"
      />
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
                  color: rgb(
                    var(--v-theme-on-surface),
                    var(--v-disabled-opacity)
                  );
                "
              >
                Loading data...
              </p>
            </v-row>
          </v-col>
        </v-row>
      </template>
    </v-col>
    <v-col lg="4">
      <v-card elevation="1">
        <v-card-title> Upload </v-card-title>

        <v-card-text>
          <v-file-input
            style="max-width: 300px"
            density="compact"
            class="align-self-start"
            accept=".S"
            label="Upload Testprogram"
            persistent-hint
            hint="Accepted files are Assemblyfiles"
            :disabled="testprogramName === DEFAULT_TESTPROGRAM"
            @update:model-value="fileUpload"
          />
        </v-card-text>
      </v-card>
    </v-col>
  </v-row>

  <ConfirmDialog
    :is-active="showDeleteTestprogramConfirmDialog"
    @cancel="cancelTestprogramDelete"
    @confirm="deleteCurrentTestprogram"
    :text="`Do you really want to delete the testprogram '${props.testprogramName}'?`"
  />
</template>
