import {
  ApolloClient,
  ApolloLink,
  createHttpLink,
  InMemoryCache,
} from "@apollo/client/core";
import { onError } from "@apollo/client/link/error";
import { RetryLink } from "@apollo/client/link/retry";
import { logErrorMessages } from "@vue/apollo-util";
import Cookies from "js-cookie";
import router from "@/router/index";

// HTTP connection to the API
const httpLink = createHttpLink({
  // You should use an absolute URL here
  uri: "/graphql/backend",
});

// HTTP connection to the auth API
const authHttpLink = createHttpLink({
  // You should use an absolute URL here
  uri: "/auth/backend",
});

// csrf middleware which appends the csrf token from the csrf cookie on each request as a header
const csrfLink = new ApolloLink((operation, forward) => {
  const csrfCookie = Cookies.get("CSRF-TOKEN");

  if (!csrfCookie) {
    return forward(operation);
  }

  const csrfToken = csrfCookie.split(".")[0];

  operation.setContext(({ headers = {} }) => ({
    ...headers,
    headers: {
      "X-CSRF-TOKEN": csrfToken,
    },
  }));
  return forward(operation);
});

const csrfRetry = new RetryLink({
  delay: {
    initial: 50,
    max: 200,
    jitter: true,
  },
  attempts: {
    max: 3,
    retryIf: (error, _operation) => {
      // 403 Forbidden is sent if something is wrong with the csrf token, so we retry in case no csrf cookie was set before
      if (error.statusCode === 403) {
        return true;
      }
      return false;
    },
  },
});

// Cache implementation
const cache = new InMemoryCache();

// Global error handler
const errorLink = onError((error) => {
  // @ts-ignore
  if (error.networkError && error.networkError.statusCode === 401) {
    // Redirect unauthorized user to login
    router.push("/login");
  }

  if (process.env.NODE_ENV !== "production") {
    logErrorMessages(error);
  }
});

// Auth error handler
const authErrorLink = onError((error) => {
  if (process.env.NODE_ENV !== "production") {
    logErrorMessages(error);
  }
});

// Create the apollo client
export const apolloClient = new ApolloClient({
  link: errorLink.concat(csrfRetry).concat(csrfLink).concat(httpLink),
  cache,
});

export const authApolloClient = new ApolloClient({
  link: authErrorLink.concat(csrfRetry).concat(csrfLink).concat(authHttpLink),
  cache,
});
