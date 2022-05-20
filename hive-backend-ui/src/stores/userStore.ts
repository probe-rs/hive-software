/// Holds all user data which is inserted on successful authentication
import { defineStore } from "pinia";

export const useUserStore = defineStore("userStore", {
  state: () => ({
    username: "",
    role: null,
  }),
});
