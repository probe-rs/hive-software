<script setup lang="ts">
import {
  Role,
  type BackendMutationModifyUserArgs,
  type UserResponse,
  type BackendMutation,
  type BackendQuery,
  type BackendMutationDeleteUserArgs,
} from "@/gql/backend";

import { ref, watch, type PropType } from "vue";
import generator from "generate-password-browser";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import { useMutation } from "@vue/apollo-composable";
import gql from "graphql-tag";
import { cloneDeep } from "@apollo/client/utilities";
import SuccessSnackbar from "./SuccessSnackbar.vue";
import { computed } from "vue";

const props = defineProps({
  name: {
    type: String,
    required: true,
  },
  description: {
    type: String,
    required: true,
  },
  expiration: {
    type: String,
    required: false,
  },
});

const showRevokeConfirmDialog = ref(false);
const displayedExpirationDate = computed(() => {
  if (props.expiration) {
    const date = new Date(Date.parse(props.expiration));

    return date.toLocaleString("en-GB");
  } else {
    return "never";
  }
});

const { mutate: revokeTokenMutation, onDone: onRevokeDone } = useMutation<
  BackendMutation,
  BackendMutationRevokeTestApiTokenArgs
>(
  gql`
    mutation ($name: String!) {
      revokeTestApiToken(name: $name)
    }
  `,
  {
    update: (cache, { data }) => {
      if (!data) {
        return;
      }

      const revokeToken = data.revokeTestApiToken;

      const QUERY = gql`
        query {
          testApiTokens {
            name
            description
            expiration
          }
        }
      `;

      let cacheData: BackendQuery | null = cache.readQuery({
        query: QUERY,
      });

      if (!cacheData) {
        return;
      }

      const newTestApiTokens = cloneDeep(cacheData.testApiTokens);

      const idx = newTestApiTokens.findIndex((e: TestApiTokensResponse) => {
        return e.name === revokeToken.name;
      });

      newTestApiTokens.splice(idx, 1);

      cacheData = {
        ...cacheData,
        testApiTokens: newTestApiTokens,
      };

      cache.writeQuery({ query: QUERY, data: cacheData });
    },
  }
);

const tokenRevokeSuccess = ref(false);

onRevokeDone(() => {
  tokenRevokeSuccess.value = true;
});

function revokeToken() {
  showRevokeConfirmDialog.value = false;

  revokeTokenMutation({ name: props.name });
}
</script>

<template>
  <tr>
    <td>
      {{ props.name }}
    </td>
    <td>
      {{ props.description }}
    </td>
    <td>
      {{ displayedExpirationDate }}
    </td>
    <td class="text-right">
      <v-btn
        icon="mdi-delete"
        size="small"
        variant="text"
        color="error"
        @click="showRevokeConfirmDialog = true"
      />
    </td>
  </tr>

  <ConfirmDialog
    :is-active="showRevokeConfirmDialog"
    @cancel="showRevokeConfirmDialog = false"
    @confirm="revokeToken"
    :text="`Do you really want to revoke this token: '${props.name}'?`"
  />

  <SuccessSnackbar
    :is-success="tokenRevokeSuccess"
    @close-event="tokenRevokeSuccess = false"
    message="Successfully revoked the token"
  />
</template>
