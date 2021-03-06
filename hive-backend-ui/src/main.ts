import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import vuetify from "@/plugins/vuetify";
import pinia from "@/plugins/pinia";
import vueKonva from "vue-konva";

const app = createApp(App);

app.use(pinia);
app.use(router);
app.use(vuetify);
app.use(vueKonva);

app.mount("#app");
