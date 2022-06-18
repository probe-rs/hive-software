import type {
  Scalars,
  InputMaybe,
  Maybe,
  Exact,
  MakeMaybe,
  MakeOptional,
} from "./baseTypes";

export type BackendAuthQuery = {
  __typename?: "BackendAuthQuery";
  /** Authenticates the provided user and sends back a jwt */
  authenticateUser: UserResponse;
};

export type BackendAuthQueryAuthenticateUserArgs = {
  username: Scalars["String"];
  password: Scalars["String"];
};

export type BackendAuthMutation = {
  __typename?: "BackendAuthMutation";
  /** Log the currently authenticated user out by deleting the auth jwt cookie */
  logout: Scalars["Boolean"];
};

/** The possible roles a user can have */
export enum Role {
  Admin = "ADMIN",
  Maintainer = "MAINTAINER",
}

export type UserResponse = {
  __typename?: "UserResponse";
  username: Scalars["String"];
  role: Role;
};
