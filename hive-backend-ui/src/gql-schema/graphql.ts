/* eslint-disable */
import type { TypedDocumentNode as DocumentNode } from "@graphql-typed-document-node/core";
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = {
  [K in keyof T]: T[K];
};
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & {
  [SubKey in K]?: Maybe<T[SubKey]>;
};
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & {
  [SubKey in K]: Maybe<T[SubKey]>;
};
export type MakeEmpty<
  T extends { [key: string]: unknown },
  K extends keyof T,
> = { [_ in K]?: never };
export type Incremental<T> =
  | T
  | {
      [P in keyof T]?: P extends " $fragmentName" | "__typename" ? T[P] : never;
    };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string };
  String: { input: string; output: string };
  Boolean: { input: boolean; output: boolean };
  Int: { input: number; output: number };
  Float: { input: number; output: number };
  Upload: { input: any; output: any };
};

/** Serializable view on a [`DbToken`] struct for usage in the GraphQL API */
export type ApiTokenInfo = {
  __typename?: "ApiTokenInfo";
  description: Scalars["String"]["output"];
  expiration?: Maybe<Scalars["String"]["output"]>;
  name: Scalars["String"]["output"];
};

/** The main applications of Hive */
export enum Application {
  Monitor = "MONITOR",
  Runner = "RUNNER",
}

/** The current supported architectures of a Testprogram */
export enum Architecture {
  Arm = "ARM",
  Riscv = "RISCV",
}

export type AssignProbeResponse = {
  __typename?: "AssignProbeResponse";
  data: FlatProbeState;
  probePos: Scalars["Int"]["output"];
};

export type AssignTargetResponse = {
  __typename?: "AssignTargetResponse";
  targetName: Scalars["String"]["output"];
  targetPos: Scalars["Int"]["output"];
  tssPos: Scalars["Int"]["output"];
};

export type BackendMutation = {
  __typename?: "BackendMutation";
  /** Assigns a probe to a given position. This does only update the data in the DB. To apply the changes into the runtime use the update mutation to reinitialize the testrack */
  assignProbe: AssignProbeResponse;
  /** Assigns a target to a given position. This does only update the data in the DB. To apply the changes into the runtime use the update mutation to reinitialize the testrack */
  assignTarget: AssignTargetResponse;
  /** Change the password of the authenticated user */
  changePassword: Scalars["Boolean"]["output"];
  /** Change the username of the authenticated user */
  changeUsername: UserResponse;
  /** Creates a new test API token for use */
  createTestApiToken: Scalars["String"]["output"];
  /** Create a testprogram */
  createTestprogram: Testprogram;
  /** Create a new user */
  createUser: UserResponse;
  /** Delete a testprogram */
  deleteTestprogram: Scalars["String"]["output"];
  /** Delete a user */
  deleteUser: UserResponse;
  /** Modify a testprogram */
  modifyTestprogram: Testprogram;
  /** Modify a user */
  modifyUser: UserResponse;
  /** Manually reinitialize the hardware in the runtime */
  reinitializeHardware: Scalars["Boolean"]["output"];
  /** Revokes the test API token with the provided name */
  revokeTestApiToken: Scalars["String"]["output"];
  /** Set a testprogram as active testprogram */
  setActiveTestprogram: Scalars["String"]["output"];
};

export type BackendMutationAssignProbeArgs = {
  probePos: Scalars["Int"]["input"];
  probeState: FlatProbeStateInput;
};

export type BackendMutationAssignTargetArgs = {
  targetName: Scalars["String"]["input"];
  targetPos: Scalars["Int"]["input"];
  tssPos: Scalars["Int"]["input"];
};

export type BackendMutationChangePasswordArgs = {
  newPassword: Scalars["String"]["input"];
  oldPassword: Scalars["String"]["input"];
};

export type BackendMutationChangeUsernameArgs = {
  username: Scalars["String"]["input"];
};

export type BackendMutationCreateTestApiTokenArgs = {
  description: Scalars["String"]["input"];
  expiration?: InputMaybe<Scalars["String"]["input"]>;
  name: Scalars["String"]["input"];
};

export type BackendMutationCreateTestprogramArgs = {
  testprogramName: Scalars["String"]["input"];
};

export type BackendMutationCreateUserArgs = {
  password: Scalars["String"]["input"];
  role: Role;
  username: Scalars["String"]["input"];
};

export type BackendMutationDeleteTestprogramArgs = {
  testprogramName: Scalars["String"]["input"];
};

export type BackendMutationDeleteUserArgs = {
  username: Scalars["String"]["input"];
};

export type BackendMutationModifyTestprogramArgs = {
  codeFiles: Array<Scalars["Upload"]["input"]>;
  testprogramName: Scalars["String"]["input"];
};

export type BackendMutationModifyUserArgs = {
  newPassword?: InputMaybe<Scalars["String"]["input"]>;
  newRole?: InputMaybe<Role>;
  newUsername?: InputMaybe<Scalars["String"]["input"]>;
  oldUsername: Scalars["String"]["input"];
};

export type BackendMutationRevokeTestApiTokenArgs = {
  name: Scalars["String"]["input"];
};

export type BackendMutationSetActiveTestprogramArgs = {
  testprogramName: Scalars["String"]["input"];
};

export type BackendQuery = {
  __typename?: "BackendQuery";
  /** Get the currently active testprogram */
  activeTestprogram: Scalars["String"]["output"];
  /** Return the log data of the provided application (either runner or monitor) */
  applicationLog: Array<Scalars["String"]["output"]>;
  /** The current probes assigned to testchannels */
  assignedProbes: Array<FlatProbeState>;
  /** The current targets assigned to connected daughterboards */
  assignedTargets: Array<Maybe<Array<FlatTargetState>>>;
  /** Get all avaialable testprograms */
  availableTestprograms: Array<Testprogram>;
  /** The currently connected daughterboards */
  connectedDaughterboards: Array<Scalars["Boolean"]["output"]>;
  /** The currently connected debug probes */
  connectedProbes: Array<ProbeInfo>;
  /** The currently connected TSS */
  connectedTss: Array<Scalars["Boolean"]["output"]>;
  /** List the registered users */
  registeredUsers: Array<UserResponse>;
  /** Search the supported targets by Hive */
  searchSupportedTargets: Array<Scalars["String"]["output"]>;
  /** Get information about the system on which Hive runs */
  systemInfo: SystemInfo;
  /** Get all currently active test API tokens */
  testApiTokens: Array<ApiTokenInfo>;
  /** Get the provided testprogram and its sourcecode as base64 */
  testprogram: FullTestProgramResponse;
};

export type BackendQueryApplicationLogArgs = {
  application: Application;
  level: LogLevel;
};

export type BackendQuerySearchSupportedTargetsArgs = {
  search: Scalars["String"]["input"];
};

export type BackendQueryTestprogramArgs = {
  testprogramName: Scalars["String"]["input"];
};

export type DiskInfo = {
  __typename?: "DiskInfo";
  free: Scalars["Int"]["output"];
  total: Scalars["Int"]["output"];
};

/** Flattened version of [`ProbeState`] to use it in graphql api */
export type FlatProbeState = {
  __typename?: "FlatProbeState";
  data?: Maybe<ProbeInfo>;
  state: State;
};

/** Flattened version of [`ProbeState`] to use it in graphql api */
export type FlatProbeStateInput = {
  data?: InputMaybe<ProbeInfoInput>;
  state: State;
};

/** Flattened version of [`TargetInfo`] to use it in graphql api */
export type FlatTargetInfo = {
  __typename?: "FlatTargetInfo";
  flashMessage: Scalars["String"]["output"];
  flashStatus: ResultEnum;
  name: Scalars["String"]["output"];
};

/** Flattened version of [`TargetState`] to use it in graphql api */
export type FlatTargetState = {
  __typename?: "FlatTargetState";
  data?: Maybe<FlatTargetInfo>;
  state: State;
};

export type FullTestProgramResponse = {
  __typename?: "FullTestProgramResponse";
  codeArm: Scalars["String"]["output"];
  codeRiscv: Scalars["String"]["output"];
  testprogram: Testprogram;
};

/** Wrapper for [`log::Level`] to use it in graphql api */
export enum LogLevel {
  Debug = "DEBUG",
  Error = "ERROR",
  Info = "INFO",
  Trace = "TRACE",
  Warn = "WARN",
}

export type MemoryInfo = {
  __typename?: "MemoryInfo";
  free: Scalars["Int"]["output"];
  total: Scalars["Int"]["output"];
};

/** Information on a probe attached to Hive */
export type ProbeInfo = {
  __typename?: "ProbeInfo";
  identifier: Scalars["String"]["output"];
  serialNumber?: Maybe<Scalars["String"]["output"]>;
};

/** Information on a probe attached to Hive */
export type ProbeInfoInput = {
  identifier: Scalars["String"]["input"];
  serialNumber?: InputMaybe<Scalars["String"]["input"]>;
};

export enum ResultEnum {
  Error = "ERROR",
  Ok = "OK",
}

/** The possible roles a user can have */
export enum Role {
  /** Can create / delete users */
  Admin = "ADMIN",
  Maintainer = "MAINTAINER",
}

export enum State {
  Known = "KNOWN",
  NotConnected = "NOT_CONNECTED",
  Unknown = "UNKNOWN",
}

/** System information of the System running this application */
export type SystemInfo = {
  __typename?: "SystemInfo";
  averageLoad: Scalars["Float"]["output"];
  controller: Scalars["String"]["output"];
  cores: Scalars["Int"]["output"];
  disk: DiskInfo;
  hostname: Scalars["String"]["output"];
  memory: MemoryInfo;
  os: Scalars["String"]["output"];
  soc: Scalars["String"]["output"];
};

export type Testprogram = {
  __typename?: "Testprogram";
  name: Scalars["String"]["output"];
  testprogramArm: TestprogramArchitecture;
  testprogramRiscv: TestprogramArchitecture;
};

/** The sub-instance of [`Testprogram`] which contains all architecture specific functionality */
export type TestprogramArchitecture = {
  __typename?: "TestprogramArchitecture";
  architecture: Architecture;
  compileMessage: Scalars["String"]["output"];
  status: TestprogramStatus;
};

export enum TestprogramStatus {
  CompileFailure = "COMPILE_FAILURE",
  NotInitialized = "NOT_INITIALIZED",
  Ok = "OK",
}

export type UserResponse = {
  __typename?: "UserResponse";
  role: Role;
  username: Scalars["String"]["output"];
};

export type ChangeUserNameMutationVariables = Exact<{
  username: Scalars["String"]["input"];
}>;

export type ChangeUserNameMutation = {
  __typename?: "BackendMutation";
  changeUsername: { __typename?: "UserResponse"; username: string; role: Role };
};

export type RegisteredUsersQueryVariables = Exact<{ [key: string]: never }>;

export type RegisteredUsersQuery = {
  __typename?: "BackendQuery";
  registeredUsers: Array<{
    __typename?: "UserResponse";
    username: string;
    role: Role;
  }>;
};

export type ChangePasswordMutationVariables = Exact<{
  oldPassword: Scalars["String"]["input"];
  newPassword: Scalars["String"]["input"];
}>;

export type ChangePasswordMutation = {
  __typename?: "BackendMutation";
  changePassword: boolean;
};

export type RevokeTestApiTokenMutationVariables = Exact<{
  name: Scalars["String"]["input"];
}>;

export type RevokeTestApiTokenMutation = {
  __typename?: "BackendMutation";
  revokeTestApiToken: string;
};

export type TestApiTokensQueryVariables = Exact<{ [key: string]: never }>;

export type TestApiTokensQuery = {
  __typename?: "BackendQuery";
  testApiTokens: Array<{
    __typename?: "ApiTokenInfo";
    name: string;
    description: string;
    expiration?: string | null;
  }>;
};

export type AssignedAndConnectedProbesQueryVariables = Exact<{
  [key: string]: never;
}>;

export type AssignedAndConnectedProbesQuery = {
  __typename?: "BackendQuery";
  assignedProbes: Array<{
    __typename?: "FlatProbeState";
    state: State;
    data?: {
      __typename?: "ProbeInfo";
      identifier: string;
      serialNumber?: string | null;
    } | null;
  }>;
  connectedProbes: Array<{
    __typename?: "ProbeInfo";
    identifier: string;
    serialNumber?: string | null;
  }>;
};

export type AssignProbeMutationVariables = Exact<{
  probePos: Scalars["Int"]["input"];
  probeState: FlatProbeStateInput;
}>;

export type AssignProbeMutation = {
  __typename?: "BackendMutation";
  assignProbe: {
    __typename?: "AssignProbeResponse";
    probePos: number;
    data: {
      __typename?: "FlatProbeState";
      state: State;
      data?: {
        __typename?: "ProbeInfo";
        identifier: string;
        serialNumber?: string | null;
      } | null;
    };
  };
};

export type AssignedProbesOverviewQueryVariables = Exact<{
  [key: string]: never;
}>;

export type AssignedProbesOverviewQuery = {
  __typename?: "BackendQuery";
  assignedProbes: Array<{
    __typename?: "FlatProbeState";
    state: State;
    data?: {
      __typename?: "ProbeInfo";
      identifier: string;
      serialNumber?: string | null;
    } | null;
  }>;
};

export type AssignedProbesQueryVariables = Exact<{ [key: string]: never }>;

export type AssignedProbesQuery = {
  __typename?: "BackendQuery";
  assignedProbes: Array<{
    __typename?: "FlatProbeState";
    state: State;
    data?: {
      __typename?: "ProbeInfo";
      identifier: string;
      serialNumber?: string | null;
    } | null;
  }>;
  connectedProbes: Array<{
    __typename?: "ProbeInfo";
    identifier: string;
    serialNumber?: string | null;
  }>;
};

export type ReinitializeHardwareMutationVariables = Exact<{
  [key: string]: never;
}>;

export type ReinitializeHardwareMutation = {
  __typename?: "BackendMutation";
  reinitializeHardware: boolean;
};

export type SystemInfoQueryVariables = Exact<{ [key: string]: never }>;

export type SystemInfoQuery = {
  __typename?: "BackendQuery";
  systemInfo: {
    __typename?: "SystemInfo";
    controller: string;
    soc: string;
    hostname: string;
    cores: number;
    os: string;
    averageLoad: number;
    memory: { __typename?: "MemoryInfo"; total: number; free: number };
    disk: { __typename?: "DiskInfo"; total: number; free: number };
  };
};

export type SearchSupportedTargetsQueryVariables = Exact<{
  search: Scalars["String"]["input"];
}>;

export type SearchSupportedTargetsQuery = {
  __typename?: "BackendQuery";
  searchSupportedTargets: Array<string>;
};

export type AssignTargetMutationVariables = Exact<{
  tssPos: Scalars["Int"]["input"];
  targetPos: Scalars["Int"]["input"];
  targetName: Scalars["String"]["input"];
}>;

export type AssignTargetMutation = {
  __typename?: "BackendMutation";
  assignTarget: {
    __typename?: "AssignTargetResponse";
    tssPos: number;
    targetPos: number;
    targetName: string;
  };
};

export type AssignedTargetsQueryVariables = Exact<{ [key: string]: never }>;

export type AssignedTargetsQuery = {
  __typename?: "BackendQuery";
  assignedTargets: Array<Array<{
    __typename?: "FlatTargetState";
    state: State;
    data?: {
      __typename?: "FlatTargetInfo";
      name: string;
      flashStatus: ResultEnum;
      flashMessage: string;
    } | null;
  }> | null>;
};

export type AvailableTestProgramsQueryVariables = Exact<{
  [key: string]: never;
}>;

export type AvailableTestProgramsQuery = {
  __typename?: "BackendQuery";
  availableTestprograms: Array<{
    __typename?: "Testprogram";
    name: string;
    testprogramArm: {
      __typename?: "TestprogramArchitecture";
      architecture: Architecture;
      status: TestprogramStatus;
      compileMessage: string;
    };
    testprogramRiscv: {
      __typename?: "TestprogramArchitecture";
      architecture: Architecture;
      status: TestprogramStatus;
      compileMessage: string;
    };
  }>;
};

export type TestProgramQueryVariables = Exact<{
  testprogramName: Scalars["String"]["input"];
}>;

export type TestProgramQuery = {
  __typename?: "BackendQuery";
  testprogram: {
    __typename?: "FullTestProgramResponse";
    codeArm: string;
    codeRiscv: string;
    testprogram: {
      __typename?: "Testprogram";
      name: string;
      testprogramArm: {
        __typename?: "TestprogramArchitecture";
        architecture: Architecture;
        status: TestprogramStatus;
        compileMessage: string;
      };
      testprogramRiscv: {
        __typename?: "TestprogramArchitecture";
        architecture: Architecture;
        status: TestprogramStatus;
        compileMessage: string;
      };
    };
  };
};

export type DeleteTestProgramMutationVariables = Exact<{
  testprogramName: Scalars["String"]["input"];
}>;

export type DeleteTestProgramMutation = {
  __typename?: "BackendMutation";
  deleteTestprogram: string;
};

export type ModifyTestProgramMutationVariables = Exact<{
  testprogramName: Scalars["String"]["input"];
  codeFiles: Array<Scalars["Upload"]["input"]> | Scalars["Upload"]["input"];
}>;

export type ModifyTestProgramMutation = {
  __typename?: "BackendMutation";
  modifyTestprogram: {
    __typename?: "Testprogram";
    name: string;
    testprogramArm: {
      __typename?: "TestprogramArchitecture";
      architecture: Architecture;
      status: TestprogramStatus;
      compileMessage: string;
    };
    testprogramRiscv: {
      __typename?: "TestprogramArchitecture";
      architecture: Architecture;
      status: TestprogramStatus;
      compileMessage: string;
    };
  };
};

export type ModifyUserMutationVariables = Exact<{
  oldUsername: Scalars["String"]["input"];
  newUsername?: InputMaybe<Scalars["String"]["input"]>;
  newPassword?: InputMaybe<Scalars["String"]["input"]>;
  newRole?: InputMaybe<Role>;
}>;

export type ModifyUserMutation = {
  __typename?: "BackendMutation";
  modifyUser: { __typename?: "UserResponse"; username: string; role: Role };
};

export type DeleteUserMutationVariables = Exact<{
  username: Scalars["String"]["input"];
}>;

export type DeleteUserMutation = {
  __typename?: "BackendMutation";
  deleteUser: { __typename?: "UserResponse"; username: string; role: Role };
};

export type ConnectedTssAndTargetsQueryVariables = Exact<{
  [key: string]: never;
}>;

export type ConnectedTssAndTargetsQuery = {
  __typename?: "BackendQuery";
  connectedTss: Array<boolean>;
  assignedTargets: Array<Array<{
    __typename?: "FlatTargetState";
    state: State;
  }> | null>;
};

export type CreateTestApiTokenMutationVariables = Exact<{
  name: Scalars["String"]["input"];
  description: Scalars["String"]["input"];
  expiration?: InputMaybe<Scalars["String"]["input"]>;
}>;

export type CreateTestApiTokenMutation = {
  __typename?: "BackendMutation";
  createTestApiToken: string;
};

export type ApplicationLogQueryVariables = Exact<{
  application: Application;
  level: LogLevel;
}>;

export type ApplicationLogQuery = {
  __typename?: "BackendQuery";
  applicationLog: Array<string>;
};

export type AvailableAndActiveTestProgramsQueryVariables = Exact<{
  [key: string]: never;
}>;

export type AvailableAndActiveTestProgramsQuery = {
  __typename?: "BackendQuery";
  activeTestprogram: string;
  availableTestprograms: Array<{ __typename?: "Testprogram"; name: string }>;
};

export type SetActiveTestProgramMutationVariables = Exact<{
  testprogramName: Scalars["String"]["input"];
}>;

export type SetActiveTestProgramMutation = {
  __typename?: "BackendMutation";
  setActiveTestprogram: string;
};

export type ActiveTestProgramQueryVariables = Exact<{ [key: string]: never }>;

export type ActiveTestProgramQuery = {
  __typename?: "BackendQuery";
  activeTestprogram: string;
};

export type CreateTestProgramMutationVariables = Exact<{
  testprogramName: Scalars["String"]["input"];
}>;

export type CreateTestProgramMutation = {
  __typename?: "BackendMutation";
  createTestprogram: {
    __typename?: "Testprogram";
    name: string;
    testprogramArm: {
      __typename?: "TestprogramArchitecture";
      architecture: Architecture;
      status: TestprogramStatus;
      compileMessage: string;
    };
    testprogramRiscv: {
      __typename?: "TestprogramArchitecture";
      architecture: Architecture;
      status: TestprogramStatus;
      compileMessage: string;
    };
  };
};

export type AvailableTestprogramsOverviewQueryVariables = Exact<{
  [key: string]: never;
}>;

export type AvailableTestprogramsOverviewQuery = {
  __typename?: "BackendQuery";
  availableTestprograms: Array<{
    __typename?: "Testprogram";
    name: string;
    testprogramArm: {
      __typename?: "TestprogramArchitecture";
      architecture: Architecture;
      status: TestprogramStatus;
      compileMessage: string;
    };
    testprogramRiscv: {
      __typename?: "TestprogramArchitecture";
      architecture: Architecture;
      status: TestprogramStatus;
      compileMessage: string;
    };
  }>;
};

export type CreateUserMutationVariables = Exact<{
  username: Scalars["String"]["input"];
  password: Scalars["String"]["input"];
  role: Role;
}>;

export type CreateUserMutation = {
  __typename?: "BackendMutation";
  createUser: { __typename?: "UserResponse"; username: string; role: Role };
};

export const ChangeUserNameDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "ChangeUserName" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "username" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "changeUsername" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "username" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "username" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "username" } },
                { kind: "Field", name: { kind: "Name", value: "role" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  ChangeUserNameMutation,
  ChangeUserNameMutationVariables
>;
export const RegisteredUsersDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "RegisteredUsers" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "registeredUsers" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "username" } },
                { kind: "Field", name: { kind: "Name", value: "role" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  RegisteredUsersQuery,
  RegisteredUsersQueryVariables
>;
export const ChangePasswordDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "ChangePassword" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "oldPassword" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "newPassword" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "changePassword" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "oldPassword" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "oldPassword" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "newPassword" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "newPassword" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  ChangePasswordMutation,
  ChangePasswordMutationVariables
>;
export const RevokeTestApiTokenDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "RevokeTestApiToken" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: { kind: "Variable", name: { kind: "Name", value: "name" } },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "revokeTestApiToken" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "name" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "name" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  RevokeTestApiTokenMutation,
  RevokeTestApiTokenMutationVariables
>;
export const TestApiTokensDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "TestApiTokens" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "testApiTokens" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "name" } },
                { kind: "Field", name: { kind: "Name", value: "description" } },
                { kind: "Field", name: { kind: "Name", value: "expiration" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<TestApiTokensQuery, TestApiTokensQueryVariables>;
export const AssignedAndConnectedProbesDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "AssignedAndConnectedProbes" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "assignedProbes" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "state" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "data" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "identifier" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "serialNumber" },
                      },
                    ],
                  },
                },
              ],
            },
          },
          {
            kind: "Field",
            name: { kind: "Name", value: "connectedProbes" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "identifier" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "serialNumber" },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  AssignedAndConnectedProbesQuery,
  AssignedAndConnectedProbesQueryVariables
>;
export const AssignProbeDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "AssignProbe" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "probePos" },
          },
          type: {
            kind: "NonNullType",
            type: { kind: "NamedType", name: { kind: "Name", value: "Int" } },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "probeState" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "FlatProbeStateInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "assignProbe" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "probePos" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "probePos" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "probeState" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "probeState" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "probePos" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "data" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      { kind: "Field", name: { kind: "Name", value: "state" } },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "data" },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "identifier" },
                            },
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "serialNumber" },
                            },
                          ],
                        },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<AssignProbeMutation, AssignProbeMutationVariables>;
export const AssignedProbesOverviewDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "AssignedProbesOverview" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "assignedProbes" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "state" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "data" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "identifier" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "serialNumber" },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  AssignedProbesOverviewQuery,
  AssignedProbesOverviewQueryVariables
>;
export const AssignedProbesDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "AssignedProbes" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "assignedProbes" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "state" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "data" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "identifier" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "serialNumber" },
                      },
                    ],
                  },
                },
              ],
            },
          },
          {
            kind: "Field",
            name: { kind: "Name", value: "connectedProbes" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "identifier" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "serialNumber" },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<AssignedProbesQuery, AssignedProbesQueryVariables>;
export const ReinitializeHardwareDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "ReinitializeHardware" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "reinitializeHardware" },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  ReinitializeHardwareMutation,
  ReinitializeHardwareMutationVariables
>;
export const SystemInfoDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "SystemInfo" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "systemInfo" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "controller" } },
                { kind: "Field", name: { kind: "Name", value: "soc" } },
                { kind: "Field", name: { kind: "Name", value: "hostname" } },
                { kind: "Field", name: { kind: "Name", value: "cores" } },
                { kind: "Field", name: { kind: "Name", value: "os" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "memory" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      { kind: "Field", name: { kind: "Name", value: "total" } },
                      { kind: "Field", name: { kind: "Name", value: "free" } },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "disk" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      { kind: "Field", name: { kind: "Name", value: "total" } },
                      { kind: "Field", name: { kind: "Name", value: "free" } },
                    ],
                  },
                },
                { kind: "Field", name: { kind: "Name", value: "averageLoad" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<SystemInfoQuery, SystemInfoQueryVariables>;
export const SearchSupportedTargetsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "SearchSupportedTargets" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "search" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "searchSupportedTargets" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "search" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "search" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  SearchSupportedTargetsQuery,
  SearchSupportedTargetsQueryVariables
>;
export const AssignTargetDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "AssignTarget" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "tssPos" },
          },
          type: {
            kind: "NonNullType",
            type: { kind: "NamedType", name: { kind: "Name", value: "Int" } },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "targetPos" },
          },
          type: {
            kind: "NonNullType",
            type: { kind: "NamedType", name: { kind: "Name", value: "Int" } },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "targetName" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "assignTarget" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "tssPos" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "tssPos" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "targetPos" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "targetPos" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "targetName" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "targetName" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "tssPos" } },
                { kind: "Field", name: { kind: "Name", value: "targetPos" } },
                { kind: "Field", name: { kind: "Name", value: "targetName" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  AssignTargetMutation,
  AssignTargetMutationVariables
>;
export const AssignedTargetsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "AssignedTargets" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "assignedTargets" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "state" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "data" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      { kind: "Field", name: { kind: "Name", value: "name" } },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "flashStatus" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "flashMessage" },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  AssignedTargetsQuery,
  AssignedTargetsQueryVariables
>;
export const AvailableTestProgramsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "AvailableTestPrograms" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "availableTestprograms" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "name" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "testprogramArm" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "architecture" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "status" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "compileMessage" },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "testprogramRiscv" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "architecture" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "status" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "compileMessage" },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  AvailableTestProgramsQuery,
  AvailableTestProgramsQueryVariables
>;
export const TestProgramDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "TestProgram" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "testprogramName" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "testprogram" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "testprogramName" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "testprogramName" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "testprogram" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      { kind: "Field", name: { kind: "Name", value: "name" } },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "testprogramArm" },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "architecture" },
                            },
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "status" },
                            },
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "compileMessage" },
                            },
                          ],
                        },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "testprogramRiscv" },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "architecture" },
                            },
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "status" },
                            },
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "compileMessage" },
                            },
                          ],
                        },
                      },
                    ],
                  },
                },
                { kind: "Field", name: { kind: "Name", value: "codeArm" } },
                { kind: "Field", name: { kind: "Name", value: "codeRiscv" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<TestProgramQuery, TestProgramQueryVariables>;
export const DeleteTestProgramDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "DeleteTestProgram" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "testprogramName" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "deleteTestprogram" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "testprogramName" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "testprogramName" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  DeleteTestProgramMutation,
  DeleteTestProgramMutationVariables
>;
export const ModifyTestProgramDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "ModifyTestProgram" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "testprogramName" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "codeFiles" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "ListType",
              type: {
                kind: "NonNullType",
                type: {
                  kind: "NamedType",
                  name: { kind: "Name", value: "Upload" },
                },
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "modifyTestprogram" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "testprogramName" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "testprogramName" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "codeFiles" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "codeFiles" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "name" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "testprogramArm" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "architecture" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "status" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "compileMessage" },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "testprogramRiscv" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "architecture" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "status" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "compileMessage" },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  ModifyTestProgramMutation,
  ModifyTestProgramMutationVariables
>;
export const ModifyUserDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "ModifyUser" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "oldUsername" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "newUsername" },
          },
          type: { kind: "NamedType", name: { kind: "Name", value: "String" } },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "newPassword" },
          },
          type: { kind: "NamedType", name: { kind: "Name", value: "String" } },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "newRole" },
          },
          type: { kind: "NamedType", name: { kind: "Name", value: "Role" } },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "modifyUser" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "oldUsername" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "oldUsername" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "newUsername" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "newUsername" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "newPassword" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "newPassword" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "newRole" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "newRole" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "username" } },
                { kind: "Field", name: { kind: "Name", value: "role" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<ModifyUserMutation, ModifyUserMutationVariables>;
export const DeleteUserDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "DeleteUser" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "username" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "deleteUser" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "username" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "username" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "username" } },
                { kind: "Field", name: { kind: "Name", value: "role" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<DeleteUserMutation, DeleteUserMutationVariables>;
export const ConnectedTssAndTargetsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "ConnectedTssAndTargets" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          { kind: "Field", name: { kind: "Name", value: "connectedTss" } },
          {
            kind: "Field",
            name: { kind: "Name", value: "assignedTargets" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "state" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  ConnectedTssAndTargetsQuery,
  ConnectedTssAndTargetsQueryVariables
>;
export const CreateTestApiTokenDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "CreateTestApiToken" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: { kind: "Variable", name: { kind: "Name", value: "name" } },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "description" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "expiration" },
          },
          type: { kind: "NamedType", name: { kind: "Name", value: "String" } },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "createTestApiToken" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "name" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "name" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "description" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "description" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "expiration" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "expiration" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  CreateTestApiTokenMutation,
  CreateTestApiTokenMutationVariables
>;
export const ApplicationLogDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "ApplicationLog" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "application" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "Application" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "level" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "LogLevel" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "applicationLog" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "application" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "application" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "level" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "level" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<ApplicationLogQuery, ApplicationLogQueryVariables>;
export const AvailableAndActiveTestProgramsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "AvailableAndActiveTestPrograms" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "availableTestprograms" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "name" } },
              ],
            },
          },
          { kind: "Field", name: { kind: "Name", value: "activeTestprogram" } },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  AvailableAndActiveTestProgramsQuery,
  AvailableAndActiveTestProgramsQueryVariables
>;
export const SetActiveTestProgramDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "SetActiveTestProgram" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "testprogramName" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "setActiveTestprogram" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "testprogramName" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "testprogramName" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  SetActiveTestProgramMutation,
  SetActiveTestProgramMutationVariables
>;
export const ActiveTestProgramDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "ActiveTestProgram" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          { kind: "Field", name: { kind: "Name", value: "activeTestprogram" } },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  ActiveTestProgramQuery,
  ActiveTestProgramQueryVariables
>;
export const CreateTestProgramDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "CreateTestProgram" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "testprogramName" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "createTestprogram" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "testprogramName" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "testprogramName" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "name" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "testprogramArm" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "architecture" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "status" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "compileMessage" },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "testprogramRiscv" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "architecture" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "status" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "compileMessage" },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  CreateTestProgramMutation,
  CreateTestProgramMutationVariables
>;
export const AvailableTestprogramsOverviewDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "AvailableTestprogramsOverview" },
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "availableTestprograms" },
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "name" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "testprogramArm" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "architecture" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "status" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "compileMessage" },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "testprogramRiscv" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "architecture" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "status" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "compileMessage" },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  AvailableTestprogramsOverviewQuery,
  AvailableTestprogramsOverviewQueryVariables
>;
export const CreateUserDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "CreateUser" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "username" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "password" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "String" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: { kind: "Variable", name: { kind: "Name", value: "role" } },
          type: {
            kind: "NonNullType",
            type: { kind: "NamedType", name: { kind: "Name", value: "Role" } },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "createUser" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "username" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "username" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "password" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "password" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "role" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "role" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                { kind: "Field", name: { kind: "Name", value: "username" } },
                { kind: "Field", name: { kind: "Name", value: "role" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<CreateUserMutation, CreateUserMutationVariables>;
