/// Holds all app configuration which needs to be persistent accross Sessions. The data is held in the Localstorage of the client.
import { defineStore } from "pinia";
import { AppTheme } from "@/plugins/vuetify";

export const useAppConfig = defineStore("appConfig", {
  state: () => ({
    theme: AppTheme.Light,
  }),
  actions: {
    toggleTheme() {
      this.theme =
        this.theme == AppTheme.Light ? AppTheme.Dark : AppTheme.Light;
    },
  },
  persist: true,
});
