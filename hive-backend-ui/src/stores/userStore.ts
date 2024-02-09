/// Holds all user data which is inserted on successful authentication
import type { Role } from "@/gql-schema/graphql";

import { defineStore } from "pinia";

export const useUserStore = defineStore("userStore", {
  state: () => ({
    username: "",
    role: null as Role | null,
  }),
  persist: true,
});
