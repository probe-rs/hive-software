<script setup lang="ts">
import { ref } from "vue";
import { VAceEditor } from "vue3-ace-editor";

import 'ace-builds/src-noconflict/mode-assembly_x86';
import 'ace-builds/src-noconflict/theme-dracula';
import 'ace-builds/src-noconflict/theme-kuroir';
import { computed } from "@vue/reactivity";
import { useAppConfig } from "@/stores/appConfig";
import { AppTheme } from "@/plugins/vuetify";

const code = ref(JSON.stringify({ text: "some code ....", to: false }))
const appConfig = useAppConfig();
const editorTheme = computed(() => {
    if (appConfig.theme === AppTheme.Light) {
        return "kuroir"
    }
    return "dracula"
})
</script>

<template>
    <VAceEditor style="height: 300px" :options="{ fontSize: 20, readOnly: false }" wrap v-model:value="code"
        :theme="editorTheme" lang="assembly_x86" />
</template>