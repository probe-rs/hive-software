<script setup lang="ts">
import { computed, ref, type Ref } from "vue";
import Terminal from "@/components/Terminal.vue";
import { useQuery } from "@vue/apollo-composable";
import { gql } from "@/gql-schema";
import { Application, LogLevel } from "@/gql-schema/graphql";
import c from "ansi-colors";

const logLevels = [
  LogLevel.Error,
  LogLevel.Warn,
  LogLevel.Info,
  LogLevel.Debug,
];

type LogEntry = {
  timestamp: string;
  level: string;
  module: string;
  message: string;
};

const selectedLogLevel: Ref<LogLevel> = ref(LogLevel.Info);
const selectedApplication = ref(0);

const selectedApplicationString = computed(() => {
  switch (selectedApplication.value) {
    case 0:
      return Application.Monitor;
    case 1:
      return Application.Runner;
    default:
      return Application.Monitor;
  }
});

const { result } = useQuery(
  gql(`
    query ApplicationLog ($application: Application!, $level: LogLevel!) {
      applicationLog(application: $application, level: $level)
    }
  `),
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
          case LogLevel.Error:
            return c.red(c.bold("error:"));
          case LogLevel.Warn:
            return c.yellow(c.bold("warn: "));
          case LogLevel.Info:
            return c.blue(c.bold("info: "));
          case LogLevel.Debug:
            return c.magenta(c.bold("debug:"));
          default:
            return "";
        }
      })();

      text += `${logEntry.timestamp} ${logLevelColored} ${c.italic(logEntry.module)} ${logEntry.message}\n`;
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
      Level: {{ selectedLogLevel }}
      <v-menu activator="parent">
        <v-list>
          <v-list-item
            v-for="level in logLevels"
            :key="level"
            :value="level"
            @click="selectedLogLevel = level"
          >
            <v-list-item-title>{{ level }}</v-list-item-title>
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
