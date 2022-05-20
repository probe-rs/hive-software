<script setup lang="ts">
import { useLazyQuery, useQuery } from "@vue/apollo-composable";
import { computed } from "@vue/reactivity";
import gql from "graphql-tag";
import { ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useUserStore } from "@/stores/userStore";

const router = useRouter();
const userStore = useUserStore();

const showPassword = ref(false);
const username = ref("");
const password = ref("");
const queryLoaded = ref(false);

const { loading, result, error, refetch, load } = useLazyQuery(
  gql`
    query ($username: String!, $password: String!) {
      authenticateUser(username: $username, password: $password) {
        username
        role
      }
    }
  `,
  { username: "", password: "" },
  { clientId: "auth", fetchPolicy: "no-cache" },
);

const isError = computed(() => {
  if (error.value) {
    return true;
  }
  return false;
});

watch(result, (newResult) => {
  // User is successfully authenticated, redirect and save user
  userStore.username = newResult.authenticateUser.username;
  userStore.role = newResult.authenticateUser.role;

  router.push("/");
});

function submitLogin() {
  if (!queryLoaded.value) {
    load(undefined, { username: username.value, password: password.value });
    queryLoaded.value = true;
    return;
  }

  refetch({ username: username.value, password: password.value });
}
</script>

<template>
  <v-card elevation="1">
    <v-card-title>
      <h3 style="font-family: Poppins">Hive Backend</h3>
    </v-card-title>
    <v-card-content>
      <v-text-field
        v-model="username"
        variant="underlined"
        density="compact"
        label="Username"
        :error="isError"
        @keydown.enter="submitLogin"
      />
      <v-text-field
        v-model="password"
        :append-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'"
        variant="underlined"
        density="compact"
        :type="showPassword ? 'text' : 'password'"
        label="Password"
        :error="isError"
        :error-messages="isError ? 'Wrong username or password' : undefined"
        @click:append="showPassword = !showPassword"
        @keydown.enter="submitLogin"
      />
    </v-card-content>
    <v-card-actions>
      <v-spacer />
      <v-btn color="secondary" variant="text" @click="submitLogin">Login</v-btn>
    </v-card-actions>

    <v-overlay v-model="loading" contained class="align-center justify-center">
      <v-progress-circular size="80" color="secondary" indeterminate />
    </v-overlay>
  </v-card>
</template>
