<script setup lang="ts">
import type { BackendAuthMutation } from "@/gql-schema/backendAuth";

import { onMounted, ref } from "vue";
import AppSettings from "@/components/AppSettings.vue";
import Navigation from "@/components/Navigation.vue";
import ErrorSnackbar from "@/components/ErrorSnackbar.vue";
import { useMutation } from "@vue/apollo-composable";
import gql from "graphql-tag";
import { useRouter } from "vue-router";
import { useUserStore } from "@/stores/userStore";
import { APOLLO_ERROR } from "@/plugins/apollo";
import hiveIcon from "@/components/icons/hive.vue";

const router = useRouter();
const userStore = useUserStore();
const showNavigation = ref(true);
const showError = ref(false);
const errorMessage = ref("");

onMounted(() => {
  document.addEventListener(APOLLO_ERROR, (e) => {
    //@ts-expect-error
    errorMessage.value = `Error: ${e.detail}`;
    showError.value = true;
  });
});

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
      <hiveIcon />
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
      <v-tooltip location="bottom end" origin="top center" activator="parent"
        >Log out</v-tooltip
      >
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
    <v-container fluid>
      <slot />
    </v-container>
  </v-main>

  <ErrorSnackbar
    :is-error="showError"
    :message="errorMessage"
    @close-event="showError = false"
  />
</template>
