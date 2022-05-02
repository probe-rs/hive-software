<script setup lang="ts">
const menu = [
  {
    title: "Overview",
    icon: "mdi-checkbook",
    url: "/",
  },
  {
    title: "Details",
    icon: "mdi-anvil",
    url: "/details",
  },
  {
    title: "Testprograms",
    icon: "mdi-desktop-classic",
    url: "/testprograms",
  },
];
</script>

<template>
  <v-list density="compact">
    <v-list-item v-for="link in menu" :key="link.title" :to="link.url" link>
      <v-list-item-avatar start>
        <v-icon>{{ link.icon }}</v-icon>
      </v-list-item-avatar>
      <v-list-item-title>{{ link.title }}</v-list-item-title>
    </v-list-item>
  </v-list>
</template>
