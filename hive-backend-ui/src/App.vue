<script setup lang="ts">
import { RouterView } from "vue-router";
import { useAppConfig } from "@/stores/appConfig";
import MenuFrame from "./components/MenuFrame.vue";
import { onMounted } from "vue";
import { useServerData } from "./stores/serverData";

const appConfig = useAppConfig();
const serverData = useServerData();

onMounted(async () => {
  await serverData.initStore();
});
</script>

<template>
  <v-app :theme="appConfig.theme">
    <MenuFrame>
      <RouterView />
    </MenuFrame>
  </v-app>
</template>

<style>
/* Custom scrollbar from https://dev.to/xtrp/how-to-create-a-beautiful-custom-scrollbar-for-your-site-in-plain-css-1mjg */
::-webkit-scrollbar {
  width: 20px;
}

::-webkit-scrollbar-track {
  background-color: transparent;
}

::-webkit-scrollbar-thumb {
  background-color: #d6dee1;
  border-radius: 20px;
  border: 6px solid transparent;
  background-clip: content-box;
}

::-webkit-scrollbar-thumb:hover {
  background-color: #a8bbbf;
}
</style>