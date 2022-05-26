<script setup lang="ts">
import { computed, ref, type Ref } from 'vue';
import Terminal from '@/components/Terminal.vue';
import { useQuery } from '@vue/apollo-composable';
import gql from 'graphql-tag';

enum LogLevel {
    ERROR = "ERROR",
    WARN = "WARN",
    INFO = "INFO",
    DEBUG = "DEBUG",
}

const logLevels = [LogLevel.ERROR, LogLevel.WARN, LogLevel.INFO, LogLevel.DEBUG];

const selectedLogLevel: Ref<LogLevel> = ref(LogLevel.INFO);
const selectedApplication = ref(0);

const selectedApplicationString = computed(() => {
    switch (selectedApplication.value) {
        case 0:
            return "MONITOR";
        case 1: return "RUNNER";
    }
})

const { result } = useQuery(gql`
    query ($application: String!, $level: String!) {
        applicationLog(application: $application, level: $level)
    }
`, () => ({
    application: selectedApplicationString.value,
    level: selectedLogLevel.value
}), {
    pollInterval: 10000,
});

const terminalText = computed(() => {
    if (result.value) {
        return result.value.applicationLog.join("");
    }
    return "Loading data...\n";
});

</script>

<template>
    <v-toolbar color="surface" elevation="1">
        <v-icon size="25" class="mr-2" icon="mdi-console" />

        <v-toolbar-title>Logs</v-toolbar-title>

        <template v-slot:extension>
            <v-tabs v-model="selectedApplication" color="secondary">
                <v-tab> MONITOR </v-tab>
                <v-tab> RUNNER </v-tab>
            </v-tabs>

            <v-spacer></v-spacer>
            <v-btn color="success">Export</v-btn>
        </template>

        <v-spacer />

        <v-btn>
            Level: {{ selectedLogLevel }}
            <v-menu activator="parent">
                <v-list>
                    <v-list-item v-for="level in logLevels" :key="level" :value="level"
                        @click="selectedLogLevel = level">
                        <v-list-item-title>{{ level }}</v-list-item-title>
                    </v-list-item>
                </v-list>
            </v-menu>
        </v-btn>
    </v-toolbar>

    <v-row class="pt-4 pb-4">
        <v-col>
            <Terminal :content="terminalText" />
        </v-col>
    </v-row>
</template>
