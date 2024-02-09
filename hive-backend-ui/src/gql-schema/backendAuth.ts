import type { Scalars } from "./graphql";

export type BackendAuthQuery = {
  __typename?: "BackendAuthQuery";
  /** Authenticates the provided user and sends back a jwt */
  authenticateUser: UserResponse;
};

export type BackendAuthQueryAuthenticateUserArgs = {
  username: Scalars["String"]["input"];
  password: Scalars["String"]["input"];
};

export type BackendAuthMutation = {
  __typename?: "BackendAuthMutation";
  /** Log the currently authenticated user out by deleting the auth jwt cookie */
  logout: Scalars["Boolean"]["input"];
};

/** The possible roles a user can have */
export enum Role {
  Admin = "ADMIN",
  Maintainer = "MAINTAINER",
}

export type UserResponse = {
  __typename?: "UserResponse";
  username: Scalars["String"]["output"];
  role: Role;
};
