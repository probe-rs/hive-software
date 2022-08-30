<script setup lang="ts">
import { computed, ref, type Ref } from "vue";
import Terminal from "@/components/Terminal.vue";
import { useQuery } from "@vue/apollo-composable";
import gql from "graphql-tag";
import type { BackendQuery } from "@/gql/backend";
import c from "ansi-colors"

enum LogLevel {
  ERROR = "ERROR",
  WARN = "WARN",
  INFO = "INFO",
  DEBUG = "DEBUG",
}

const logLevels = [
  LogLevel.ERROR,
  LogLevel.WARN,
  LogLevel.INFO,
  LogLevel.DEBUG,
];

type LogEntry = {
  timestamp: string,
  level: string,
  module: string,
  message: string,
}

const selectedLogLevel: Ref<LogLevel> = ref(LogLevel.INFO);
const selectedApplication = ref(0);

const selectedApplicationString = computed(() => {
  switch (selectedApplication.value) {
    case 0:
      return "MONITOR";
    case 1:
      return "RUNNER";
    default:
      return "MONITOR";
  }
});

const { result } = useQuery<BackendQuery>(
  gql`
    query ($application: String!, $level: String!) {
      applicationLog(application: $application, level: $level)
    }
  `,
  () => ({
    application: selectedApplicationString.value,
    level: selectedLogLevel.value,
  }),
  {
    pollInterval: 10000,
  },
);

const terminalText = computed<string>(() => {
  if (result.value) {
    // Explicitly enable string coloring (For whatever reason this is needed to actually get ansi sequences in the string)
    c.enabled = true;

    let text = "";

    result.value.applicationLog.forEach((line) => {
      const logEntry: LogEntry = JSON.parse(line);

      const logLevelColored = (() => {
        switch (logEntry.level) {
          case LogLevel.ERROR:
            return c.red(c.bold("error:"));
          case LogLevel.WARN:
            return c.yellow(c.bold("warn: "));
          case LogLevel.INFO:
            return c.blue(c.bold("info: "));
          case LogLevel.DEBUG:
            return c.magenta(c.bold("debug:"));
          default:
            return ""
        }
      })();

      text += (`${logEntry.timestamp} ${logLevelColored} ${c.italic(logEntry.module)} ${logEntry.message}\n`);
    });

    return text;
  }
  return "Loading data...\n";
});

function exportLog() {
  const url = window.URL.createObjectURL(
    new Blob([terminalText.value], { type: "text/plain" }),
  );

  const link = document.createElement("a");
  link.href = url;
  link.setAttribute(
    "download",
    `${selectedApplicationString.value.toLowerCase()}_log.txt`,
  );

  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}
</script>

<template>
  <v-toolbar color="surface" elevation="1">
    <v-icon size="25" class="ml-2" icon="mdi-console" />

    <v-toolbar-title>Logs</v-toolbar-title>

    <template v-slot:extension>
      <v-tabs v-model="selectedApplication" color="secondary">
        <v-tab> MONITOR </v-tab>
        <v-tab> RUNNER </v-tab>
      </v-tabs>

      <v-spacer></v-spacer>
      <v-btn color="success" @click="exportLog">Export</v-btn>
    </template>

    <v-spacer />

    <v-btn>
      Level: {{  selectedLogLevel  }}
      <v-menu activator="parent">
        <v-list>
          <v-list-item v-for="level in logLevels" :key="level" :value="level" @click="selectedLogLevel = level">
            <v-list-item-title>{{  level  }}</v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>
    </v-btn>
  </v-toolbar>

  <v-row class="pt-4 pb-4">
    <v-col>
      <Terminal :content="terminalText" :scrollToBottom="true" />
    </v-col>
  </v-row>
</template>
