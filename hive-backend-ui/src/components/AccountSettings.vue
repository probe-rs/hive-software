<script setup lang="ts">
import { useUserStore } from "@/stores/userStore";
import { useMutation } from "@vue/apollo-composable";
import gql from "graphql-tag";
import { ref, watch } from "vue";
import SuccessSnackbar from "@/components/SuccessSnackbar.vue";

const emit = defineEmits(["closeEvent"]);

const userStore = useUserStore();

const dataChanged = ref(false);
const localUsername = ref(userStore.username);

const isError = ref(false);
const errorMessage = ref("")

const { loading, mutate: changeUsername, onError: onUsernameChangeError, onDone: onUsernameChangeDone } = useMutation(
  gql`
    mutation ($username: String!) {
      changeUsername(username: $username) {
        username
      }
    }
  `,
  {
    fetchPolicy: "no-cache",
  },
);

watch(localUsername, () => {
  dataChanged.value = true;
});

async function submitData() {
  if (dataChanged.value) {
    changeUsername({ username: localUsername.value });
  } else {
    emit("closeEvent");
  }
}

onUsernameChangeError((error) => {
  isError.value = true;
  errorMessage.value = error.message;
});

onUsernameChangeDone(({ data }) => {
  const newUsername = data.changeUsername.username;
  localUsername.value = newUsername;
  userStore.username = newUsername;

  dataChanged.value = false;

  emit("closeEvent");
})

</script>

<template>
  <v-card style="min-width: 50vw">
    <v-card-title class="text-h5 grey lighten-2">
      Account Settings
    </v-card-title>

    <v-card-text>
      <v-form>
        <v-text-field v-model="localUsername" label="Username" :error="isError"
          :error-messages="isError ? errorMessage : undefined" />
        <v-text-field label="Role" v-model="userStore.role" disabled />
        <v-btn color="info"> Change Password </v-btn>
      </v-form>
    </v-card-text>

    <v-divider></v-divider>

    <v-card-actions>
      <v-btn color="error" text @click="$emit('closeEvent')"> Cancel </v-btn>
      <v-spacer></v-spacer>
      <v-btn color="success" text @click="submitData"> Save and Exit </v-btn>
    </v-card-actions>

    <!--Replace with loading save and exit button once available in vuetify-->
    <v-overlay v-model="loading" contained class="align-center justify-center">
      <v-progress-circular size="80" color="secondary" indeterminate />
    </v-overlay>
  </v-card>
</template>
