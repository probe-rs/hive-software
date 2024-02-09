<script setup lang="ts">
import { ref, watch, type PropType } from "vue";
import generator from "generate-password-browser";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import { useMutation } from "@vue/apollo-composable";
import { gql } from "@/gql-schema";
import { Role } from "@/gql-schema/graphql";
import { cloneDeep } from "@apollo/client/utilities";
import SuccessSnackbar from "./SuccessSnackbar.vue";

const props = defineProps({
  username: {
    type: String,
    required: true,
  },
  role: {
    type: String as PropType<Role>,
    required: true,
  },
});

const REGISTERED_USERS_QUERY = gql(`
        query RegisteredUsers {
          registeredUsers {
            username
            role
          }
        }
      `);

const dataChanged = ref(false);
const showPasswordConfirmDialog = ref(false);
const showDeleteConfirmDialog = ref(false);

const modifiedUsername = ref(props.username);
const modifiedRole = ref(props.role);

watch(modifiedUsername, () => {
  dataChanged.value = true;
});

watch(modifiedRole, () => {
  dataChanged.value = true;
});

const { mutate: mutateUser, onDone: onModifyDone } = useMutation(
  gql(`
    mutation ModifyUser (
      $oldUsername: String!
      $newUsername: String
      $newPassword: String
      $newRole: Role
    ) {
      modifyUser(
        oldUsername: $oldUsername
        newUsername: $newUsername
        newPassword: $newPassword
        newRole: $newRole
      ) {
        username
        role
      }
    }
  `),
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const modifyUser = data.modifyUser;

      let cacheData = cache.readQuery({
        query: REGISTERED_USERS_QUERY,
      });

      if (!cacheData) {
        return;
      }

      const newRegisteredUsers = cloneDeep(cacheData.registeredUsers);

      const idx = newRegisteredUsers.findIndex((e) => {
        return e.username === modifyUser.username;
      });

      newRegisteredUsers[idx] = modifyUser;

      cacheData = {
        ...cacheData,
        registeredUsers: newRegisteredUsers,
      };

      cache.writeQuery({ query: REGISTERED_USERS_QUERY, data: cacheData });
    },
  },
);

const userModifySuccess = ref(false);

onModifyDone(() => {
  dataChanged.value = false;
  userModifySuccess.value = true;
});

function modifyUser() {
  mutateUser({
    oldUsername: props.username,
    newUsername: modifiedUsername.value,
    newRole: modifiedRole.value,
  });
}

function changePassword() {
  showPasswordConfirmDialog.value = false;

  const generatedPassword = generator.generate({
    length: 24,
    numbers: true,
    symbols: true,
    strict: true,
  });

  navigator.clipboard.writeText(generatedPassword);

  mutateUser({ oldUsername: props.username, newPassword: generatedPassword });
}

const { mutate: deleteUserMutation, onDone: onDeleteDone } = useMutation(
  gql(`
    mutation DeleteUser ($username: String!) {
      deleteUser(username: $username) {
        username
        role
      }
    }
  `),
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const deleteUser = data.deleteUser;

      let cacheData = cache.readQuery({
        query: REGISTERED_USERS_QUERY,
      });

      if (!cacheData) {
        return;
      }

      const newRegisteredUsers = cloneDeep(cacheData.registeredUsers);

      const idx = newRegisteredUsers.findIndex((e) => {
        return e.username === deleteUser.username;
      });

      console.log("found idx: ", idx);
      console.log("current registered: ", JSON.stringify(newRegisteredUsers));

      newRegisteredUsers.splice(idx, 1);

      console.log("afterSplice: ", JSON.stringify(newRegisteredUsers));

      cacheData = {
        ...cacheData,
        registeredUsers: newRegisteredUsers,
      };

      cache.writeQuery({ query: REGISTERED_USERS_QUERY, data: cacheData });

      console.log(
        "new cached query: ",
        cache.readQuery({
          query: REGISTERED_USERS_QUERY,
        }),
      );
    },
  },
);

const userDeleteSuccess = ref(false);

onDeleteDone(() => {
  userDeleteSuccess.value = true;
});

function deleteUser() {
  showDeleteConfirmDialog.value = false;

  deleteUserMutation({ username: props.username });
}
</script>

<template>
  <tr>
    <td>
      <input
        type="text"
        label="Username"
        variant="plain"
        density="compact"
        v-model="modifiedUsername"
      />
    </td>
    <td>
      <p style="cursor: pointer">
        {{ modifiedRole }}

        <v-menu activator="parent">
          <v-list density="compact">
            <v-list-item
              :disabled="modifiedRole === Role.Admin"
              :style="modifiedRole === Role.Admin ? '' : 'cursor:pointer'"
            >
              <v-list-item-title @click="modifiedRole = Role.Admin"
                >ADMIN</v-list-item-title
              >
            </v-list-item>
            <v-list-item
              :disabled="modifiedRole === Role.Maintainer"
              :style="modifiedRole === Role.Maintainer ? '' : 'cursor:pointer'"
            >
              <v-list-item-title @click="modifiedRole = Role.Maintainer"
                >MAINTAINER</v-list-item-title
              >
            </v-list-item>
          </v-list>
        </v-menu>
      </p>
    </td>
    <td class="text-right">
      <v-btn
        size="small"
        variant="text"
        color="info"
        @click="showPasswordConfirmDialog = true"
        >Reset Password
      </v-btn>
      <v-btn
        icon="mdi-delete"
        size="small"
        variant="text"
        color="error"
        @click="showDeleteConfirmDialog = true"
      />
      <v-btn
        v-if="dataChanged"
        size="small"
        variant="text"
        color="success"
        @click="modifyUser"
        >Apply Changes
      </v-btn>
    </td>
  </tr>

  <ConfirmDialog
    :is-active="showPasswordConfirmDialog"
    @cancel="showPasswordConfirmDialog = false"
    @confirm="changePassword"
    :text="`Do you really want to change the password of the user '${props.username}'?`"
  />

  <ConfirmDialog
    :is-active="showDeleteConfirmDialog"
    @cancel="showDeleteConfirmDialog = false"
    @confirm="deleteUser"
    :text="`Do you really want to delete the user '${props.username}'?`"
  />

  <SuccessSnackbar
    :is-success="userModifySuccess"
    @close-event="userModifySuccess = false"
    message="Successfully modified user, any regenerated passwords have been copied to clipboard"
  />
  <SuccessSnackbar
    :is-success="userDeleteSuccess"
    @close-event="userDeleteSuccess = false"
    message="Successfully deleted user"
  />
</template>

<style scoped>
/* Style username input/edit field */
input[type="text"]:focus-visible {
  outline: none !important;
  border-bottom: 2px solid;
}
</style>
