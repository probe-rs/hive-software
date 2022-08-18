<script setup lang="ts">
import { ref, onMounted, onBeforeMount, onUnmounted, watch, toRefs } from "vue";
import { Terminal } from "xterm";
import "xterm/css/xterm.css";
import { FitAddon } from "xterm-addon-fit";
// @ts-ignore
import * as XtermWebfont from "xterm-webfont";

const props = defineProps({
  content: {
    type: String,
    required: true,
  },
  scrollToBottom: {
    type: Boolean,
    required: true,
  },
});

const { content, scrollToBottom } = toRefs(props);

const terminalParent = ref(null);
const terminal = new Terminal({
  allowProposedApi: false,
  disableStdin: true,
  fontFamily: "Ubuntu Mono",
  fontSize: 20,
  theme: {
    background: "#282a36",
  },
  convertEol: true,
});
const terminalFit = new FitAddon();
terminal.loadAddon(terminalFit);
terminal.loadAddon(new XtermWebfont());
terminal.write(content.value);

watch(content, (newVal) => {
  terminal.reset();
  terminal.write(newVal);

  if (scrollToBottom.value) {
    terminal.scrollToBottom();
  }
});

onBeforeMount(() => {
  window.addEventListener("resize", updateTerminalSize);
});

onMounted(async () => {
  // @ts-ignore
  await terminal.loadWebfontAndOpen(terminalParent.value!);
  updateTerminalSize();
});

onUnmounted(() => {
  window.removeEventListener("resize", updateTerminalSize);
});

function updateTerminalSize() {
  terminalFit.fit();
}
</script>

<template>
  <div class="terminal" ref="terminalParent" />
</template>

<style>
.xterm {
  padding-left: 0.5rem;
}

.terminal {
  width: 100%;
  height: 100%;
  overflow: hidden;
}
</style>
