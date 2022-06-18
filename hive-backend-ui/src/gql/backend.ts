import type {
  Scalars,
  InputMaybe,
  Maybe,
  Exact,
  MakeMaybe,
  MakeOptional,
} from "./baseTypes";

/** The main applications of Hive */
export enum Application {
  Monitor = "MONITOR",
  Runner = "RUNNER",
}

export type AssignProbeResponse = {
  __typename?: "AssignProbeResponse";
  probePos: Scalars["Int"];
  data: FlatProbeState;
};

export type AssignTargetResponse = {
  __typename?: "AssignTargetResponse";
  tssPos: Scalars["Int"];
  targetPos: Scalars["Int"];
  targetName: Scalars["String"];
};

export type BackendMutation = {
  __typename?: "BackendMutation";
  /** Assigns a target to a given position. This does only update the data in the DB. To apply the changes into the runtime use the update mutation to reinitialize the testrack */
  assignTarget: AssignTargetResponse;
  /** Assigns a probe to a given position. This does only update the data in the DB. To apply the changes into the runtime use the update mutation to reinitialize the testrack */
  assignProbe: AssignProbeResponse;
  /** Manually reinitialize the hardware in the runtime */
  reinitializeHardware: Scalars["Boolean"];
  /** Change the username of the authenticated user */
  changeUsername: UserResponse;
  /** Change the password of the authenticated user */
  changePassword: Scalars["Boolean"];
  /** Create a new user */
  createUser: UserResponse;
  /** Delete a user */
  deleteUser: UserResponse;
  /** Modify a user */
  modifyUser: UserResponse;
};

export type BackendMutationAssignTargetArgs = {
  tssPos: Scalars["Int"];
  targetPos: Scalars["Int"];
  targetName: Scalars["String"];
};

export type BackendMutationAssignProbeArgs = {
  probePos: Scalars["Int"];
  probeState: FlatProbeStateInput;
};

export type BackendMutationChangeUsernameArgs = {
  username: Scalars["String"];
};

export type BackendMutationChangePasswordArgs = {
  oldPassword: Scalars["String"];
  newPassword: Scalars["String"];
};

export type BackendMutationCreateUserArgs = {
  username: Scalars["String"];
  password: Scalars["String"];
  role: Role;
};

export type BackendMutationDeleteUserArgs = {
  username: Scalars["String"];
};

export type BackendMutationModifyUserArgs = {
  oldUsername: Scalars["String"];
  newRole?: InputMaybe<Role>;
  newUsername?: InputMaybe<Scalars["String"]>;
  newPassword?: InputMaybe<Scalars["String"]>;
};

export type BackendQuery = {
  __typename?: "BackendQuery";
  /** The currently connected daughterboards */
  connectedDaughterboards: Array<Scalars["Boolean"]>;
  /** The currently connected TSS */
  connectedTss: Array<Scalars["Boolean"]>;
  /** The current targets assigned to connected daughterboards */
  assignedTargets: Array<Maybe<Array<FlatTargetState>>>;
  /** The current probes assigned to testchannels */
  assignedProbes: Array<FlatProbeState>;
  /** Search the supported targets by Hive */
  searchSupportedTargets: Array<Scalars["String"]>;
  /** The currently connected debug probes */
  connectedProbes: Array<ProbeInfo>;
  /** Return the log data of the provided application (either runner or monitor) */
  applicationLog: Array<Scalars["String"]>;
  /** List the registered users */
  registeredUsers: Array<UserResponse>;
};

export type BackendQuerySearchSupportedTargetsArgs = {
  search: Scalars["String"];
};

export type BackendQueryApplicationLogArgs = {
  application: Application;
  level: LogLevel;
};

/** Flattened version of [`ProbeState`] to use it in graphql api */
export type FlatProbeState = {
  __typename?: "FlatProbeState";
  state: State;
  data?: Maybe<ProbeInfo>;
};

/** Flattened version of [`ProbeState`] to use it in graphql api */
export type FlatProbeStateInput = {
  state: State;
  data?: InputMaybe<ProbeInfoInput>;
};

/** Flattened version of [`TargetState`] to use it in graphql api */
export type FlatTargetState = {
  __typename?: "FlatTargetState";
  state: State;
  data?: Maybe<TargetInfo>;
};

/** Wrapper for [`log::Level`] to use it in graphql api */
export enum LogLevel {
  Error = "ERROR",
  Warn = "WARN",
  Info = "INFO",
  Debug = "DEBUG",
  Trace = "TRACE",
}

/** Information on a probe attached to Hive */
export type ProbeInfo = {
  __typename?: "ProbeInfo";
  identifier: Scalars["String"];
  serialNumber?: Maybe<Scalars["String"]>;
};

/** Information on a probe attached to Hive */
export type ProbeInfoInput = {
  identifier: Scalars["String"];
  serialNumber?: InputMaybe<Scalars["String"]>;
};

/** The possible roles a user can have */
export enum Role {
  Admin = "ADMIN",
  Maintainer = "MAINTAINER",
}

export enum State {
  Known = "KNOWN",
  Unknown = "UNKNOWN",
  NotConnected = "NOT_CONNECTED",
}

export type TargetInfo = {
  __typename?: "TargetInfo";
  name: Scalars["String"];
};

export type UserResponse = {
  __typename?: "UserResponse";
  username: Scalars["String"];
  role: Role;
};
