<script setup lang="ts">
import CodeEditor from "@/components/CodeEditor.vue";
import Terminal from "@/components/Terminal.vue";
import { useMutation, useQuery } from "@vue/apollo-composable";
import { computed, ref, toRefs } from "vue";
import { gql } from "@/gql-schema";
import {
  Architecture,
  type FullTestProgramResponse,
} from "@/gql-schema/graphql";
import { watch, type PropType } from "vue";
import * as base64 from "base-64";
import { cloneDeep } from "@apollo/client/utilities";
import ConfirmDialog from "./ConfirmDialog.vue";

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

const AVAILABLE_TESTPROGRAMS_QUERY = gql(`
query AvailableTestPrograms{
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

const { selectedArchitecture, testprogramName, deleteTestprogramEvent } =
  toRefs(props);

const emit = defineEmits([
  "testprogramDeleted",
  "testprogramNotDeleted",
  "codeEdited",
]);

const testprogram = ref<FullTestProgramResponse | null>(null);
const codeArm = ref("");
const codeRiscv = ref("");

const {
  loading,
  onResult: onTestprogramResult,
  refetch: refetchTestprogram,
} = useQuery(
  gql(`
    query TestProgram ($testprogramName: String!) {
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
  `),
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
    default:
      throw new Error("Found unknown architecture in switch case");
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
    default:
      throw new Error("Encountered unknown architecture in switch case");
  }
});

watch(deleteTestprogramEvent, (isDelete) => {
  if (isDelete) {
    showDeleteTestprogramConfirmDialog.value = true;
  }
});

const { mutate: deleteTestprogram, onDone: onTestprogramDeleted } = useMutation(
  gql(`
    mutation DeleteTestProgram ($testprogramName: String!) {
      deleteTestprogram(testprogramName: $testprogramName)
    }
  `),
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const deleteTestprogram = data.deleteTestprogram;

      let cacheData = cache.readQuery({
        query: AVAILABLE_TESTPROGRAMS_QUERY,
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

      cache.writeQuery({
        query: AVAILABLE_TESTPROGRAMS_QUERY,
        data: cacheData,
      });
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

/*
TODO: Testprogram modification is currently not implemented

const {
  mutate: modifyTestprogram,
  onDone: onTestprogramModified,
  onError: OnTestprogramModifyError,
  loading: testprogramModifyLoading,
} = useMutation(
  gql(`
    mutation ModifyTestProgram ($testprogramName: String!, $codeFiles: [Upload!]!) {
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
  `),
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const modifyTestprogram = data.modifyTestprogram;

      let cacheData = cache.readQuery({
        query: AVAILABLE_TESTPROGRAMS_QUERY,
      });

      if (!cacheData) {
        return;
      }

      const newAvailableTestprograms = cloneDeep(
        cacheData.availableTestprograms
      );
      const modifyIndex = newAvailableTestprograms.findIndex((e) => {
        e.name === modifyTestprogram.name;
      });

      newAvailableTestprograms[modifyIndex] = modifyTestprogram;

      cacheData = {
        ...cacheData,
        availableTestprograms: newAvailableTestprograms,
      };

      cache.writeQuery({
        query: AVAILABLE_TESTPROGRAMS_QUERY,
        data: cacheData,
      });
    },
  }
);*/

async function fileUpload(file: File | Array<File>) {
  if (Array.isArray(file)) {
    code.value.value = await file[0].text();
  } else {
    code.value.value = await file.text();
  }
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
            @update:modelValue="fileUpload"
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
