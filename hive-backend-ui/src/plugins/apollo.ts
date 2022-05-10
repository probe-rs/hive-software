import {
    ApolloClient,
    createHttpLink,
    InMemoryCache,
} from "@apollo/client/core";
import { onError } from '@apollo/client/link/error'
import { logErrorMessages } from '@vue/apollo-util'

// HTTP connection to the API
const httpLink = createHttpLink({
    // You should use an absolute URL here
    uri: "/graphql/backend",
});

// Cache implementation
const cache = new InMemoryCache();

// Global error handler
const link = onError(error => {
    if (process.env.NODE_ENV !== 'production') {
        logErrorMessages(error)
    }
})

// Create the apollo client
export const apolloClient = new ApolloClient({
    link: link.concat(httpLink),
    cache,
});