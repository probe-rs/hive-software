<script setup lang="ts">
import { RouterView, useRoute } from "vue-router";
import { useAppConfig } from "@/stores/appConfig";
import MenuFrame from "./components/MenuFrame.vue";
import { computed } from "@vue/reactivity";

const appConfig = useAppConfig();

const currentRoute = computed(()=>{
  return useRoute().name
})
</script>

<template>
  <v-app :theme="appConfig.theme">
  <template v-if="currentRoute === 'login'">
      <RouterView />
  </template>
  <template v-else>
    <MenuFrame>
      <RouterView />
    </MenuFrame>
    </template>
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
