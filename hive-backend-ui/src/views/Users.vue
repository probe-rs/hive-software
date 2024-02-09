<script setup lang="ts">
import User from "@/components/User.vue";
import { useMutation, useQuery } from "@vue/apollo-composable";
import { gql } from "@/gql-schema";
import { Role } from "@/gql-schema/graphql";
import { computed, ref } from "vue";
import SuccessSnackbar from "@/components/SuccessSnackbar.vue";
import generator from "generate-password-browser";
import { cloneDeep } from "@apollo/client/utilities";

const REGISTERED_USERS_QUERY = gql(`
  query RegisteredUsers {
    registeredUsers {
      username
      role
    }
  }
`);

const { result, loading } = useQuery(REGISTERED_USERS_QUERY);

const users = computed(() => {
  if (result.value) {
    return result.value.registeredUsers;
  } else {
    return [];
  }
});

const addUserDialog = ref(false);
const newUsername = ref("");
const newUserRole = ref(Role.Maintainer);
const addUserSuccess = ref(false);
const roles = ref([Role.Admin, Role.Maintainer]);

const { mutate: createUser, onDone: onCreateDone } = useMutation(
  gql(`
    mutation CreateUser ($username: String!, $password: String!, $role: Role!) {
      createUser(username: $username, password: $password, role: $role) {
        username
        role
      }
    }
  `),
  {
    update: (cache, { data: resultData }) => {
      if (!resultData) return;

      const createUser = resultData.createUser;

      let data: any = cache.readQuery({
        query: REGISTERED_USERS_QUERY,
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

      cache.writeQuery({ query: REGISTERED_USERS_QUERY, data });
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
    length: 24,
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
    <v-icon size="25" class="ml-2" icon="mdi-account" />

    <v-toolbar-title>Users</v-toolbar-title>

    <v-spacer />

    <v-btn icon @click="addUserDialog = true">
      <v-icon>mdi-account-plus</v-icon>
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
            v-for="user in users"
            :username="user.username"
            :role="user.role"
            :key="`user-${user.username}`"
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
        <v-btn color="error" variant="text" @click="closeAddUserDialog">
          Cancel
        </v-btn>
        <v-spacer></v-spacer>
        <v-btn color="success" variant="text" @click="addUser">
          Create new user
        </v-btn>
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
  </v-dialog>

  <SuccessSnackbar
    :isSuccess="addUserSuccess"
    @closeEvent="addUserSuccess = false"
    message="Successfully added new user, the generated password has been copied to clipboard."
  />
</template>
