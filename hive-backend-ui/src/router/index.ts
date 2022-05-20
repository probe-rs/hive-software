import { createRouter, createWebHistory } from "vue-router";
import Overview from "@/views/Overview.vue";
import Testprograms from "@/views/Testprograms.vue";
import NotFound from "@/views/NotFound.vue";
import Login from "@/views/Login.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "overview",
      component: Overview,
    },
    {
      path: "/login",
      name: "login",
      component: Login,
    },
    {
      path: "/testprograms",
      name: "testprograms",
      component: Testprograms,
    },
    { path: "/:pathMatch(.*)*", component: NotFound, name: "notFound" },
  ],
});

export default router;
