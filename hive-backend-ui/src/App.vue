<script setup lang="ts">
import { RouterView } from "vue-router";
import { useAppConfig } from "@/stores/appConfig";
import MenuFrame from "./components/MenuFrame.vue";

const appConfig = useAppConfig();
</script>

<template>
  <v-app :theme="appConfig.theme">
    <MenuFrame>
      <RouterView />
    </MenuFrame>
  </v-app>
</template>
