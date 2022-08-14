<script setup lang="ts">
import { computed, ref } from "vue";
import { AppTheme } from "@/plugins/vuetify";
import { useAppConfig } from "@/stores/appConfig";
import AccountSettings from "@/components/AccountSettings.vue";

const appConfig = useAppConfig();

const themeText = computed(() => {
  let text = "Dark Theme";
  let icon = "mdi-weather-night";
  if (appConfig.theme === AppTheme.Dark) {
    text = "Light Theme";
    icon = "mdi-weather-sunny";
  }
  return { text, icon };
});

const accountDialog = ref(false);
</script>

<template>
  <v-list density="compact">
    <v-list-item key="Account" @click="" prepend-icon="mdi-account">
      <v-list-item-title>Account</v-list-item-title>

      <v-dialog v-model="accountDialog" activator="parent" persistent max-width="800px"
        transition="dialog-top-transition">
        <account-settings @closeEvent="accountDialog = false" />
      </v-dialog>
    </v-list-item>
    <v-list-item key="Theme" @click="appConfig.toggleTheme" :prepend-icon="themeText.icon">
      <v-list-item-title v-text="themeText.text"></v-list-item-title>
    </v-list-item>
  </v-list>
</template>
