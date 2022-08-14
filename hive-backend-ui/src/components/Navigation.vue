<script setup lang="ts">
import { useUserStore } from "@/stores/userStore";

const menu = [
  {
    title: "Overview",
    icon: "mdi-checkbook",
    url: "/",
  },
  {
    title: "Testprograms",
    icon: "mdi-desktop-classic",
    url: "/testprograms",
  },
  {
    title: "Users",
    icon: "mdi-account",
    url: "/users",
    minRole: "ADMIN",
  },
  {
    title: "Logs",
    icon: "mdi-console",
    url: "/logs",
  },
];

// This is wrong and not reactive, use computed instead to modify base data
function isAuthorized(min_role: string | undefined): boolean {
  if (min_role) {
    const store = useUserStore();

    if (min_role === store.role) {
      return true;
    }

    return false;
  }
  return true;
}
</script>

<template>
  <v-list density="compact">
    <v-list-item v-for="link in menu" :v-if="isAuthorized(link.minRole)" :key="link.title" :to="link.url" link
      :prependIcon="link.icon">
      <v-list-item-title>{{ link.title }}</v-list-item-title>
    </v-list-item>
  </v-list>
</template>
