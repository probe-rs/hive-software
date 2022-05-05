<script setup lang="ts"></script>

<template>
  <v-card style="min-width: 50vw">
    <v-card-title class="text-h5 grey lighten-2">
      Account Settings
    </v-card-title>

    <v-card-text>
      <v-form>
        <v-text-field label="Username" />
        <v-text-field label="Role" model-value="Admin" disabled />
        <v-btn color="info"> Change Password </v-btn>
      </v-form>
    </v-card-text>

    <v-divider></v-divider>

    <v-card-actions>
      <v-spacer></v-spacer>
      <v-btn color="success" text @click="$emit('closeEvent')">
        Save and Exit
      </v-btn>
    </v-card-actions>
  </v-card>
</template>
