<script setup lang="ts">
import ApiToken from "@/components/ApiToken.vue";
import { useMutation, useQuery } from "@vue/apollo-composable";
import { gql } from "@/gql-schema";
import { computed, ref } from "vue";
import { cloneDeep } from "@apollo/client/utilities";

const TEST_API_TOKENS_QUERY = gql(`
  query TestApiTokens {
    testApiTokens {
      name
      description
      expiration
    }
  }
`);
const { result, loading } = useQuery(TEST_API_TOKENS_QUERY);

const tokens = computed(() => {
  if (result.value) {
    return result.value.testApiTokens;
  } else {
    return [];
  }
});

const generateTokenDialog = ref(false);
const newTokenName = ref("");
const newTokenDescription = ref("");
const newTokenIsTemporary = ref(false);
const newTokenExpirationDate = ref("");
const dateInputError = ref(false);
const dateInputInvalid = ref(false);
const tokenCreateSuccess = ref(false);
const token = ref("");

const dateInputErrorMessage = computed(() => {
  if (dateInputError.value) {
    if (dateInputInvalid.value) {
      return "Provided date is invalid";
    } else {
      return "Please select a expiration date for the temporary token";
    }
  }

  return null; // no error
});

const minExpirationDate = new Date(Date.now()).toISOString().slice(0, 19);

const { mutate: createToken, onDone: onCreateDone } = useMutation(
  gql(`
    mutation CreateTestApiToken ($name: String!, $description: String!, $expiration: String) {
      createTestApiToken(
        name: $name
        description: $description
        expiration: $expiration
      )
    }
  `),
  {
    update: (cache, { data: resultData }, { variables }) => {
      if (!resultData) return;
      if (!variables) return;

      let data: any = cache.readQuery({
        query: TEST_API_TOKENS_QUERY,
      });

      const newTestApiTokens = cloneDeep(data.testApiTokens);

      newTestApiTokens.push({
        name: variables.name,
        description: variables.description,
        expiration: variables.expiration,
      });

      data = {
        ...data,
        testApiTokens: newTestApiTokens,
      };

      cache.writeQuery({ query: TEST_API_TOKENS_QUERY, data });
    },
  },
);

function addToken() {
  let date = null;

  // reset errors
  dateInputError.value = false;
  dateInputInvalid.value = false;

  if (newTokenIsTemporary.value) {
    if (newTokenExpirationDate.value === "") {
      // No date selected
      dateInputError.value = true;
      return;
    }

    const parsedDate = Date.parse(newTokenExpirationDate.value);

    if (isNaN(parsedDate)) {
      // Invalid date
      dateInputInvalid.value = true;
      return;
    }

    // Transform expiration date to rfc3339 format
    // TODO currently ignores the users timezone and assumes timezone of the server
    date = new Date(parsedDate).toISOString();
  }

  createToken({
    name: newTokenName.value,
    description: newTokenDescription.value,
    expiration: date,
  });
}

onCreateDone((res) => {
  if (!res.data) return;

  token.value = res.data.createTestApiToken;

  tokenCreateSuccess.value = true;

  closeAddTokenDialog();
});

function closeAddTokenDialog() {
  newTokenName.value = "";
  newTokenDescription.value = "";
  newTokenIsTemporary.value = false;
  newTokenExpirationDate.value = "";

  generateTokenDialog.value = false;
}

function closeTokenSuccessSnackbar() {
  tokenCreateSuccess.value = false;
  token.value = "";
}
</script>

<template>
  <v-toolbar color="surface" elevation="1">
    <v-icon size="25" class="ml-2" icon="mdi-key-variant" />

    <v-toolbar-title>API Tokens</v-toolbar-title>

    <v-spacer />

    <v-btn icon @click="generateTokenDialog = true">
      <v-icon>mdi-plus</v-icon>
      <v-tooltip activator="parent" anchor="bottom end"
        >Generate Token</v-tooltip
      >
    </v-btn>
  </v-toolbar>

  <v-row class="pt-4">
    <v-col>
      <v-table v-if="!loading">
        <thead>
          <tr>
            <th class="text-left text-overline">Name</th>
            <th class="text-left text-overline">Description</th>
            <th class="text-left text-overline">Expires</th>
            <th class="text-right text-overline">Actions</th>
          </tr>
        </thead>
        <tbody>
          <ApiToken
            v-for="token in tokens"
            :name="token.name"
            :description="token.description"
            :expiration="token.expiration ?? undefined"
            :key="`token-${token.name}`"
          />
        </tbody>
      </v-table>
      <v-progress-linear v-else indeterminate color="secondary" />
    </v-col>
  </v-row>

  <v-dialog
    v-model="generateTokenDialog"
    persistent
    max-width="800px"
    transition="dialog-top-transition"
  >
    <v-card style="min-width: 50vw">
      <v-card-title class="text-h5 grey lighten-2">
        Generate Token
      </v-card-title>

      <v-card-text>
        <v-form>
          <v-text-field
            v-model="newTokenName"
            label="Token name"
            variant="underlined"
            density="compact"
          />
          <v-text-field
            v-model="newTokenDescription"
            label="description"
            variant="underlined"
            density="compact"
          />
          <v-switch
            label="Temporary token"
            v-model="newTokenIsTemporary"
            :color="newTokenIsTemporary ? 'primary' : ''"
          />
          <v-text-field
            v-if="newTokenIsTemporary"
            type="datetime-local"
            :min="minExpirationDate"
            v-model="newTokenExpirationDate"
            label="token expiration date"
            variant="underlined"
            density="compact"
            :error-messages="dateInputErrorMessage"
          />
        </v-form>
      </v-card-text>

      <v-divider></v-divider>

      <v-card-actions>
        <v-btn color="error" variant="text" @click="closeAddTokenDialog">
          Cancel
        </v-btn>
        <v-spacer></v-spacer>
        <v-btn color="success" variant="text" @click="addToken">
          Generate new token
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

  <v-snackbar v-model="tokenCreateSuccess" color="success" :timeout="-1">
    Token successfully created:
    <code style="background-color: black; padding: 4px">{{ token }}</code>
    Please copy the code. In case you loose the code you need to create a new
    token.
    <template v-slot:actions>
      <v-btn
        icon="mdi-close"
        size="small"
        variant="text"
        @click="closeTokenSuccessSnackbar"
      />
    </template>
  </v-snackbar>
</template>
