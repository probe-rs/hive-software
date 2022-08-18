<script setup lang="ts">
import { ref, watch } from "vue";
import { VAceEditor } from "vue3-ace-editor";

import "ace-builds/src-noconflict/mode-assembly_x86";
import "ace-builds/src-noconflict/theme-dracula";
import "ace-builds/src-noconflict/theme-kuroir";
import { computed, toRefs } from "@vue/reactivity";
import { useAppConfig } from "@/stores/appConfig";
import { AppTheme } from "@/plugins/vuetify";

const props = defineProps({
  code: {
    type: String,
    required: true,
  },
  readOnly: {
    type: Boolean,
    required: true,
  },
});

// type-based
const emit = defineEmits<{
  (e: "codeChange", code: string): void;
}>();

const { code, readOnly } = toRefs(props);

// The code prop is not mutable, therefore mutations happen on the mutableCode ref.
const mutableCode = ref(props.code);

watch(code, (newCode) => {
  // Update mutableCode with newCode
  mutableCode.value = newCode;
});

function onCodeChange() {
  emit("codeChange", mutableCode.value);
}

const appConfig = useAppConfig();
const editorTheme = computed(() => {
  if (appConfig.theme === AppTheme.Light) {
    return "kuroir";
  }
  return "dracula";
});
</script>

<template>
  <VAceEditor
    style="height: 100%"
    :options="{
      fontSize: 20,
      fontFamily: 'Ubuntu Mono',
      readOnly: readOnly,
    }"
    wrap
    v-model:value="mutableCode"
    :theme="editorTheme"
    lang="assembly_x86"
    @change="onCodeChange"
  />
</template>
