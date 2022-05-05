<script setup lang="ts">
import { ref, onMounted, onBeforeMount, onUnmounted } from "vue";
import { Terminal } from "xterm";
import "xterm/css/xterm.css";
import { FitAddon } from "xterm-addon-fit";

const terminalParent = ref(null);
const terminal = new Terminal({
  allowProposedApi: false,
  disableStdin: true,
  fontFamily: "Ubuntu Mono",
  fontSize: 20,
  theme: {
    background: "#282a36",
  },
});
const terminalFit = new FitAddon();
terminal.loadAddon(terminalFit);
terminal.writeln("Assembler & Linker output:");

onBeforeMount(() => {
  window.addEventListener("resize", updateTerminalSize);
});

onMounted(() => {
  terminal.open(terminalParent.value!);
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
