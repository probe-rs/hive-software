<script setup lang="ts">
import CodeEditor from "@/components/CodeEditor.vue";
import Terminal from "@/components/Terminal.vue";
import {
  Architecture,
  type BackendQuery,
  type BackendQueryTestprogramArgs,
} from "@/gql/backend";
import { useQuery } from "@vue/apollo-composable";
import { computed } from "@vue/reactivity";
import gql from "graphql-tag";
import type { PropType } from "vue";
import * as base64 from "base-64";

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
});

const { result, loading } = useQuery<BackendQuery, BackendQueryTestprogramArgs>(
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
    testprogramName: props.testprogramName,
  },
);

const code = computed(() => {
  if (!result.value) {
    return "loading code...";
  }

  switch (props.selectedArchitecture) {
    case Architecture.Arm:
      return base64.decode(result.value.testprogram.codeArm);
    case Architecture.Riscv:
      return base64.decode(result.value.testprogram.codeRiscv);
  }
});

const compileMessage = computed(() => {
  if (!result.value) {
    return "loading output...";
  }

  switch (props.selectedArchitecture) {
    case Architecture.Arm:
      return result.value.testprogram.testprogram.testprogramArm.compileMessage;
    case Architecture.Riscv:
      return result.value.testprogram.testprogram.testprogramRiscv
        .compileMessage;
  }
});
</script>

<template>
  <v-row class="pt-4">
    <v-col style="height: 55vh">
      <CodeEditor v-if="!loading" :code="code" :read-only="props.testprogramName === DEFAULT_TESTPROGRAM" />
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
                  color: rgb(
                    var(--v-theme-on-surface),
                    var(--v-disabled-opacity)
                  );
                ">
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
      <Terminal v-if="!loading" :content="compileMessage" />
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
                  color: rgb(
                    var(--v-theme-on-surface),
                    var(--v-disabled-opacity)
                  );
                ">
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
          <v-file-input style="max-width: 300px" density="compact" class="align-self-start" accept=".S"
            label="Upload Testprogram" persistent-hint hint="Accepted files are Assemblyfiles"
            :disabled="props.testprogramName === DEFAULT_TESTPROGRAM" />
        </v-card-text>
      </v-card>
    </v-col>
  </v-row>
</template>
