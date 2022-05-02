<script setup lang="ts">
import { ref } from "vue";
import CodeEditor from "@/components/CodeEditor.vue"

const items = ["Default", "Hive Standard"];
const selectedTestprogram = ref("Default");
</script>

<template>
    <v-toolbar color="surface" elevation="1">
        <v-icon size="25" class="mr-2" icon="mdi-file-code-outline" />

        <v-toolbar-title>Testprogram</v-toolbar-title>

        <v-spacer></v-spacer>

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
        <v-col>
            <CodeEditor />
        </v-col>
    </v-row>

    <v-row>
        <v-col>
            <v-file-input
    accept=".o,.S"
    label="Upload testprogram"
  />
<v-btn color="error">Delete testprogram</v-btn>
        </v-col>
    </v-row>


</template>