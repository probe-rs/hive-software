/// Holds all data which is sent by the server.
/// Will later be used by the websocket connection to automatically
/// update the state of the app on data change
import { defineStore } from "pinia";
import { apolloClient } from "@/plugins/apollo";
import gql from "graphql-tag";
import { provideApolloClient } from "@vue/apollo-composable";

provideApolloClient(apolloClient);

export const useServerData = defineStore("serverData", {
  state: () => ({
    targetData: [
      null,
      null,
      null,
      null,
      null,
      null,
      null,
      null,
    ],
    availableTss: [true, true, true, true, true, false, false, false],
  }),
  getters: {
    getConnectedShields: (state) => {
      state.targetData.filter((_, idx) => {
        return state.availableTss[idx];
      });
    },
  },
  actions: {
    async initStore() {
      const serverData = await apolloClient.query({
        query: gql`
          query {
            targetData {
              state
              data {
                name
              }
            }
          }
        `,
      });

      this.targetData = serverData.data.targetData;
    },
  },
});
