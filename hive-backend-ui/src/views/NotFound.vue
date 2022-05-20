<script setup lang="ts">
// Assets
import corro from "@/assets/ferris/corro.svg";
</script>

<template>
  <v-main>
    <v-container fluid>
      <v-row class="justify-center align-stretch align-self-center">
        <v-col md="6">
          <v-row class="pa-6">
            <v-img :src="corro" height="125" />
          </v-row>

          <v-row class="justify-center">
            <p
              style="
                text-align: center;
                color: rgb(
                  var(--v-theme-on-surface),
                  var(--v-disabled-opacity)
                );
              "
            >
              Could not find the requested source or page. Is your link correct?
            </p>
          </v-row>

          <v-row class="justify-center ma-6">
            <v-btn :to="{ name: 'overview' }"> Back to safety! </v-btn>
          </v-row>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>
