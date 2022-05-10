import { createApp, h, provide } from "vue";
import { DefaultApolloClient, provideApolloClient } from "@vue/apollo-composable";
import App from "./App.vue";
import router from "./router";
import vuetify from "@/plugins/vuetify";
import pinia from "@/plugins/pinia";
import vueKonva from "vue-konva";
import { apolloClient } from "@/plugins/apollo";

const app = createApp(App);

app.use(pinia);
app.use(router);
app.use(vuetify);
app.use(vueKonva);

provideApolloClient(apolloClient);

//app.provide(DefaultApolloClient, apolloClient);

app.mount("#app");