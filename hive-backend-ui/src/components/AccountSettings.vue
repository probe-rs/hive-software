<script setup lang="ts">
import type {
  BackendMutation,
  BackendMutationChangeUsernameArgs,
  BackendQuery,
} from "@/gql/backend";

import { useUserStore } from "@/stores/userStore";
import { cloneDeep } from "@apollo/client/utilities";
import { useMutation } from "@vue/apollo-composable";
import gql from "graphql-tag";
import { ref, watch } from "vue";
import SuccessSnackbar from "./SuccessSnackbar.vue";

const emit = defineEmits(["closeEvent"]);

const userStore = useUserStore();

const usernameChanged = ref(false);
const passwordChanged = ref(false);
const localUsername = ref(userStore.username);
const showChangePassword = ref(false);
const showPassword = ref(false);
const oldPassword = ref("");
const newPassword = ref("");
const newPasswordConfirm = ref("");

const isUsernameError = ref(false);
const usernameErrorMessage = ref("");

const isPasswordError = ref(false);
const isPasswordSuccess = ref(false);
const passwordErrorMessage = ref("");

const {
  loading,
  mutate: changeUsername,
  onError: onUsernameChangeError,
  onDone: onUsernameChangeDone,
} = useMutation<BackendMutation, BackendMutationChangeUsernameArgs>(
  gql`
    mutation ($username: String!) {
      changeUsername(username: $username) {
        username
        role
      }
    }
  `,
  {
    fetchPolicy: "no-cache",
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const changeUsername = data.changeUsername;

      const QUERY = gql`
        query {
          registeredUsers {
            username
            role
          }
        }
      `;

      let cacheData: BackendQuery | null = cache.readQuery({
        query: QUERY,
      });

      if (!cacheData) {
        return;
      }

      const newRegisteredUsers = cloneDeep(cacheData.registeredUsers);

      const idx = newRegisteredUsers.findIndex((e) => {
        e.username === userStore.username;
      });

      newRegisteredUsers[idx] = changeUsername;

      cacheData = {
        ...cacheData,
        registeredUsers: newRegisteredUsers,
      };

      cache.writeQuery<BackendQuery>({ query: QUERY, data: cacheData });
    },
  },
);

const {
  loading: passwordLoading,
  mutate: changePassword,
  onError: onPasswordChangeError,
  onDone: onPasswordChangeDone,
} = useMutation(
  gql`
    mutation ($oldPassword: String!, $newPassword: String!) {
      changePassword(oldPassword: $oldPassword, newPassword: $newPassword)
    }
  `,
  {
    fetchPolicy: "no-cache",
  },
);

watch(localUsername, () => {
  usernameChanged.value = true;
});

watch([newPassword, newPasswordConfirm, oldPassword], () => {
  passwordChanged.value = true;
});

function submitData() {
  if (usernameChanged.value) {
    changeUsername({ username: localUsername.value });
  } else {
    emit("closeEvent");
  }
}

function submitPassword() {
  if (!passwordChanged.value) {
    return;
  }

  if (newPassword.value !== newPasswordConfirm.value) {
    isPasswordError.value = true;
    passwordErrorMessage.value = "Passwords do not match";
    return;
  }

  changePassword({
    oldPassword: oldPassword.value,
    newPassword: newPassword.value,
  });
}

onUsernameChangeError((error) => {
  isUsernameError.value = true;
  usernameErrorMessage.value = error.message;
});

onPasswordChangeError((error) => {
  isPasswordError.value = true;
  passwordErrorMessage.value = error.message;
});

onUsernameChangeDone(({ data }) => {
  const newUsername = data!.changeUsername.username;
  localUsername.value = newUsername;
  userStore.username = newUsername;

  usernameChanged.value = false;

  emit("closeEvent");
});

onPasswordChangeDone(() => {
  passwordChanged.value = false;
  showChangePassword.value = false;
  isPasswordSuccess.value = true;
});
</script>

<template>
  <v-card style="min-width: 50vw">
    <v-card-title class="text-h5 grey lighten-2">
      Account Settings
    </v-card-title>

    <v-card-text>
      <v-form>
        <v-text-field v-model="localUsername" label="Username" :error="isUsernameError" variant="underlined"
          density="compact" :error-messages="isUsernameError ? usernameErrorMessage : undefined" />
        <v-text-field label="Role" v-model="userStore.role" disabled variant="underlined" density="compact" />
        <template v-if="showChangePassword">
          <v-text-field v-model="oldPassword" variant="underlined" density="compact"
            :type="showPassword ? 'text' : 'password'" label="Old Password" :error="isPasswordError" />
          <v-text-field v-model="newPassword" variant="underlined" density="compact"
            :type="showPassword ? 'text' : 'password'" label="New Password" :error="isPasswordError" />
          <v-text-field v-model="newPasswordConfirm" variant="underlined" density="compact"
            :append-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'" :type="showPassword ? 'text' : 'password'"
            label="New Password Confirm" :error="isPasswordError"
            :error-messages="isPasswordError ? passwordErrorMessage : undefined" :loading="passwordLoading"
            @click:append="showPassword = !showPassword" />
        </template>
        <v-btn v-if="!showChangePassword" color="info" @click="showChangePassword = true">
          Change Password
        </v-btn>
        <v-btn v-if="showChangePassword" color="info" @click="submitPassword">
          Confirm New Password
        </v-btn>
      </v-form>
    </v-card-text>

    <v-divider></v-divider>

    <v-card-actions>
      <v-btn color="error" variant="text" @click="$emit('closeEvent')"> Cancel </v-btn>
      <v-spacer></v-spacer>
      <v-btn color="success" variant="text" @click="submitData"> Save and Exit </v-btn>
    </v-card-actions>

    <!--Replace with loading save and exit button once available in vuetify-->
    <v-overlay v-model="loading" contained class="align-center justify-center">
      <v-progress-circular size="80" color="secondary" indeterminate />
    </v-overlay>
  </v-card>

  <SuccessSnackbar :isSuccess="isPasswordSuccess" @closeEvent="isPasswordSuccess = false"
    message="Successfully changed the password" />
</template>
