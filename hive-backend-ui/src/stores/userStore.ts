/// Holds all user data which is inserted on successful authentication
import type { Role } from "@/gql/backend";
import type { Maybe } from "@/gql/baseTypes";

import { defineStore } from "pinia";

export const useUserStore = defineStore("userStore", {
  state: () => ({
    username: "",
    role: null as Maybe<Role>,
  }),
  persist: true,
});
