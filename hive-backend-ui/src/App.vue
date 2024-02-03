<script setup lang="ts">
import { RouterView, useRoute } from "vue-router";
import { useAppConfig } from "@/stores/appConfig";
import MenuFrame from "./components/MenuFrame.vue";
import { computed } from "vue";
import { provide } from "vue";
import { ApolloClients } from "@vue/apollo-composable";
import { apolloClient, authApolloClient } from "./plugins/apollo";
import { watch } from "vue";
import { removeFragmentSpreadFromDocument } from "@apollo/client/utilities";

provide(ApolloClients, { default: apolloClient, auth: authApolloClient });

const appConfig = useAppConfig();
const route = useRoute();

const currentRoute = computed(() => {
  return route.name;
});
</script>

<template>
  <v-app :theme="appConfig.theme">
    <v-layout>
      <template v-if="currentRoute === 'login' || currentRoute === 'notFound'">
        <RouterView />
      </template>
      <template v-else>
        <MenuFrame>
          <RouterView />
        </MenuFrame>
      </template>
    </v-layout>
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
