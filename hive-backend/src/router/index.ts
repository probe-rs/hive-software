import { createRouter, createWebHistory } from "vue-router";
import Overview from "@/views/Overview.vue";
import Testprograms from "@/views/Testprograms.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "overview",
      component: Overview,
    },
    {
      path: "/testprograms",
      name: "testprograms",
      component: Testprograms,
    },
  ],
});

export default router;
