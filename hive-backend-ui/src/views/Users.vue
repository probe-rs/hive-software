<script setup lang="ts">
import type { BackendQuery } from "@/gql/backend";

import User from "@/components/User.vue";
import { useMutation, useQuery } from "@vue/apollo-composable";
import gql from "graphql-tag";
import { computed, ref } from "vue";
import SuccessSnackbar from "@/components/SuccessSnackbar.vue";
import generator from "generate-password";
import { cloneDeep } from "@apollo/client/utilities";

const { result, loading } = useQuery<BackendQuery>(gql`
  query {
    registeredUsers {
      username
      role
    }
  }
`);

const users = computed(() => {
  if (result.value) {
    return result.value.registeredUsers;
  } else {
    return [];
  }
});

const addUserDialog = ref(false);
const newUsername = ref("");
const newUserRole = ref("MAINTAINER");
const addUserSuccess = ref(false);
const roles = ref(["ADMIN", "MAINTAINER"]);

const { mutate: createUser, onDone: onCreateDone } = useMutation(
  gql`
    mutation ($username: String!, $password: String!, $role: String!) {
      createUser(username: $username, password: $password, role: $role) {
        username
        role
      }
    }
  `,
  {
    update: (cache, { data: { createUser } }) => {
      const QUERY = gql`
        query {
          registeredUsers {
            username
            role
          }
        }
      `;

      let data: any = cache.readQuery({
        query: QUERY,
      });

      const newRegisteredUsers = cloneDeep(data.registeredUsers);

      newRegisteredUsers.push({
        username: createUser.username,
        role: createUser.role,
      });

      data = {
        ...data,
        registeredUsers: newRegisteredUsers,
      };

      cache.writeQuery({ query: QUERY, data });
    },
  },
);

onCreateDone(() => {
  closeAddUserDialog();
  addUserSuccess.value = true;
});

function closeAddUserDialog() {
  addUserDialog.value = false;
  newUsername.value = "";
}

function addUser() {
  const generatedPassword = generator.generate({
    length: 12,
    numbers: true,
    symbols: true,
    strict: true,
  });

  navigator.clipboard.writeText(generatedPassword);

  createUser({
    username: newUsername.value,
    password: generatedPassword,
    role: newUserRole.value,
  });
}
</script>

<template>
  <v-toolbar color="surface" elevation="1">
    <v-icon size="25" class="mr-2" icon="mdi-account" />

    <v-toolbar-title>Users</v-toolbar-title>

    <v-spacer />

    <v-btn icon="mdi-plus" @click="addUserDialog = true">
      <v-tooltip activator="parent" anchor="bottom end">Add User</v-tooltip>
    </v-btn>
  </v-toolbar>

  <v-row class="pt-4">
    <v-col>
      <v-table v-if="!loading">
        <thead>
          <tr>
            <th class="text-left text-overline">Username</th>
            <th class="text-left text-overline">Role</th>
            <th class="text-right text-overline">Actions</th>
          </tr>
        </thead>
        <tbody>
          <User
            v-for="(user, idx) in users"
            :username="user.username"
            :role="user.role"
            :key="`user-${idx}`"
          />
        </tbody>
      </v-table>
      <v-progress-linear v-else indeterminate color="secondary" />
    </v-col>
  </v-row>

  <v-dialog
    v-model="addUserDialog"
    persistent
    max-width="800px"
    transition="dialog-top-transition"
  >
    <v-card style="min-width: 50vw">
      <v-card-title class="text-h5 grey lighten-2"> Add User </v-card-title>

      <v-card-text>
        <v-form>
          <v-text-field
            v-model="newUsername"
            label="Username"
            variant="underlined"
            density="compact"
          />
          <v-select
            v-model="newUserRole"
            label="Role"
            variant="underlined"
            :items="roles"
          ></v-select>
        </v-form>
      </v-card-text>

      <v-divider></v-divider>

      <v-card-actions>
        <v-btn color="error" text @click="closeAddUserDialog"> Cancel </v-btn>
        <v-spacer></v-spacer>
        <v-btn color="success" text @click="addUser"> Create new user </v-btn>
      </v-card-actions>

      <!--Replace with loading save and exit button once available in vuetify-->
      <v-overlay
        v-model="loading"
        contained
        class="align-center justify-center"
      >
        <v-progress-circular size="80" color="secondary" indeterminate />
      </v-overlay>
    </v-card>

    <SuccessSnackbar
      :isSuccess="addUserSuccess"
      @closeEvent="addUserSuccess = false"
      message="Successfully added new user, the generated password has been copied to clipboard."
    />
  </v-dialog>
</template>
