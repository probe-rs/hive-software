// Creates typescript definition out of the GraphQL SDL from Hive Backend

import { CodegenConfig } from "@graphql-codegen/cli";

const config: CodegenConfig = {
  schema: "schema.gql",
  documents: ["src/**/*.{ts,tsx,vue}"],
  generates: {
    "./src/gql-schema/": {
      preset: "client",
      plugins: [],
      config: {
        useTypeImports: true,
      },
      presetConfig: {
        gqlTagName: "gql",
      },
    },
  },
  ignoreNoDocuments: true,
};

export default config;
