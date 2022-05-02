<script setup lang="ts">
import { ref } from "vue";
import CodeEditor from "@/components/CodeEditor.vue";
import Terminal from "@/components/Terminal.vue";

const items = ["Default", "Hive Standard"];
const selectedTestprogram = ref("Default");
</script>

<template>
    <v-toolbar color="surface" elevation="1">
        <v-icon size="25" class="mr-2" icon="mdi-file-code-outline" />

        <v-toolbar-title>Testprogram</v-toolbar-title>

        <template v-slot:extension>
            <v-tabs color="secondary">
                <v-tab>
                    ARM
                </v-tab>
                <v-tab>
                    RISCV
                </v-tab>
            </v-tabs>
        </template>

        <v-spacer />

        <v-btn>
            {{ selectedTestprogram }}
            <v-menu activator="parent">
                <v-list>
                    <v-list-item v-for="item in items" :key="item" :value="item" @click="selectedTestprogram = item">
                        <v-list-item-title>{{ item }}</v-list-item-title>
                    </v-list-item>
                </v-list>
            </v-menu>
        </v-btn>

        <v-btn icon="mdi-plus">
            <v-tooltip activator="parent" anchor="bottom end">Add testprogram</v-tooltip>
        </v-btn>
    </v-toolbar>

    <v-row class="pt-4">
        <v-col style="height: 55vh;">
            <CodeEditor />
        </v-col>
    </v-row>

    <v-row>
        <v-col style="height: 30vh;" lg="8">
            <Terminal />
        </v-col>
        <v-col lg="4">
            <v-card elevation="1">
                <v-card-title> Manage Testprogram </v-card-title>

                <v-card-content>
                    <div class="d-flex justify-space-between">
                        <v-file-input style="max-width: 300px" density="compact" class="align-self-start" accept=".o,.S"
                            label="Upload Testprogram" />

                        <v-btn class="align-self-baseline" color="error">Delete Testprogram</v-btn>
                    </div>
                </v-card-content>
            </v-card>
        </v-col>
    </v-row>
</template>
