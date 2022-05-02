// Styles
import "@mdi/font/css/materialdesignicons.css";
import "vuetify/styles";

// Vuetify
import { createVuetify } from "vuetify";

// Custom themes
export enum AppTheme {
  Light = "hiveLightTheme",
  Dark = "hiveDarkTheme",
}

const HiveLightTheme = {
  dark: false,
  colors: {
    primary: "#00796B",
    secondary: "#004b79",
    background: "#EDEDED",
  },
};

const HiveDarkTheme = {
  dark: true,
  colors: {
    primary: "#807344",
    secondary: "#5ea9a1",
    background: "#121212",
  },
};

export default createVuetify({
  theme: {
    defaultTheme: "hiveLightTheme",
    themes: {
      hiveLightTheme: { ...HiveLightTheme },
      hiveDarkTheme: { ...HiveDarkTheme },
    },
  },
});
