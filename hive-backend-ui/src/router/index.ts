import { createRouter, createWebHashHistory } from "vue-router";
import Overview from "@/views/Overview.vue";
import Testprograms from "@/views/Testprograms.vue";
import NotFound from "@/views/NotFound.vue";
import Login from "@/views/Login.vue";
import Logs from "@/views/Logs.vue";
import Users from "@/views/Users.vue";
import ApiTokens from "@/views/ApiTokens.vue";

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
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
    {
      path: "/logs",
      name: "logs",
      component: Logs,
    },
    {
      path: "/users",
      name: "users",
      component: Users,
      beforeEnter: () => {
        /*const store = useUserStore();
        if (store.role !== "ADMIN") {
          return false
        }*/
        return true;
      },
    },
    {
      path: "/api-tokens",
      name: "api tokens",
      component: ApiTokens,
    },
    { path: "/:pathMatch(.*)*", component: NotFound, name: "notFound" },
  ],
});

export default router;
