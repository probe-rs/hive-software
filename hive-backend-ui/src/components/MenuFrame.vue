<script setup lang="ts">
import type { BackendAuthMutation } from "@/gql/backendAuth";

import { ref } from "vue";
import AppSettings from "@/components/AppSettings.vue";
import Navigation from "@/components/Navigation.vue";
import hiveLogo from "@/assets/probe-rs-icon.png";
import { useMutation } from "@vue/apollo-composable";
import gql from "graphql-tag";
import { useRouter } from "vue-router";
import { useUserStore } from "@/stores/userStore";

const router = useRouter();
const userStore = useUserStore();
const showNavigation = ref(true);

const { mutate: logout, onDone: onLogoutDone } =
  useMutation<BackendAuthMutation>(
    gql`
      mutation {
        logout
      }
    `,
    { clientId: "auth", fetchPolicy: "no-cache" },
  );

onLogoutDone(() => {
  router.push("login");
  userStore.username = "";
  userStore.role = null;
});

function toggleNavigation() {
  showNavigation.value = !showNavigation.value;
}

function resizeEvent() {
  window.dispatchEvent(new Event("resize"));
}
</script>

<template>
  <v-app-bar color="primary" app>
    <v-btn icon rounded="0" dark class="ml-1 pa-1" @click="toggleNavigation">
      <v-img :src="hiveLogo" alt="menu" />
    </v-btn>

    <p
      style="font-family: Poppins; font-size: 27pt; color: white"
      class="font-weight-bold pl-2"
    >
      Hive
    </p>

    <v-spacer></v-spacer>
    <v-menu rounded="0" anchor="bottom end" origin="auto">
      <template v-slot:activator="{ props }">
        <v-btn icon rounded="0" v-bind="props">
          <v-icon> mdi-cog </v-icon>
        </v-btn>
      </template>
      <AppSettings />
    </v-menu>
    <v-btn icon rounded="0" @click="logout">
      <v-tooltip anchor="bottom end" activator="parent">Log out</v-tooltip>
      <v-icon> mdi-logout </v-icon>
    </v-btn>
  </v-app-bar>

  <v-navigation-drawer
    clipped
    :model-value="showNavigation"
    app
    @transitionend="resizeEvent"
  >
    <Navigation />
  </v-navigation-drawer>

  <v-main>
    <v-container fluid class="fill-height">
      <slot />
    </v-container>
  </v-main>
</template>
