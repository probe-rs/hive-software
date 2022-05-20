import { createApp } from "vue";
import { provideApolloClients } from "@vue/apollo-composable";
import App from "./App.vue";
import router from "./router";
import vuetify from "@/plugins/vuetify";
import pinia from "@/plugins/pinia";
import vueKonva from "vue-konva";
import { apolloClient, authApolloClient } from "@/plugins/apollo";

const app = createApp(App);

app.use(pinia);
app.use(router);
app.use(vuetify);
app.use(vueKonva);

//provideApolloClients({ default: apolloClient, auth: authApolloClient });

app.mount("#app");
