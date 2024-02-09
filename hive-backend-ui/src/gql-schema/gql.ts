/* eslint-disable */
import * as types from "./graphql";
import type { TypedDocumentNode as DocumentNode } from "@graphql-typed-document-node/core";

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 */
const documents = {
  "\n    mutation ChangeUserName ($username: String!) {\n      changeUsername(username: $username) {\n        username\n        role\n      }\n    }\n  ":
    types.ChangeUserNameDocument,
  "\n        query RegisteredUsers{\n          registeredUsers {\n            username\n            role\n          }\n        }\n      ":
    types.RegisteredUsersDocument,
  "\n    mutation ChangePassword ($oldPassword: String!, $newPassword: String!) {\n      changePassword(oldPassword: $oldPassword, newPassword: $newPassword)\n    }\n  ":
    types.ChangePasswordDocument,
  "\n    mutation RevokeTestApiToken ($name: String!) {\n      revokeTestApiToken(name: $name)\n    }\n  ":
    types.RevokeTestApiTokenDocument,
  "\n        query TestApiTokens {\n          testApiTokens {\n            name\n            description\n            expiration\n          }\n        }\n      ":
    types.TestApiTokensDocument,
  "\n  query AssignedAndConnectedProbes {\n    assignedProbes {\n      state\n      data {\n        identifier\n        serialNumber\n      }\n    }\n    connectedProbes {\n      identifier\n      serialNumber\n    }\n  }\n":
    types.AssignedAndConnectedProbesDocument,
  "\n    mutation AssignProbe ($probePos: Int!, $probeState: FlatProbeStateInput!) {\n      assignProbe(probePos: $probePos, probeState: $probeState) {\n        probePos\n        data {\n          state\n          data {\n            identifier\n            serialNumber\n          }\n        }\n      }\n    }\n  ":
    types.AssignProbeDocument,
  "\n        query AssignedProbesOverview {\n          assignedProbes {\n            state\n            data {\n              identifier\n              serialNumber\n            }\n          }\n        }\n      ":
    types.AssignedProbesOverviewDocument,
  "\n    query AssignedProbes {\n      assignedProbes {\n        state\n        data {\n          identifier\n          serialNumber\n        }\n      }\n      connectedProbes {\n        identifier\n        serialNumber\n      }\n    }\n  ":
    types.AssignedProbesDocument,
  "\n    mutation ReinitializeHardware {\n      reinitializeHardware\n    }\n  ":
    types.ReinitializeHardwareDocument,
  "\n    query SystemInfo {\n      systemInfo {\n        controller\n        soc\n        hostname\n        cores\n        os\n        memory {\n          total\n          free\n        }\n        disk {\n          total\n          free\n        }\n        averageLoad\n      }\n    }\n  ":
    types.SystemInfoDocument,
  "\n      query SearchSupportedTargets ($search: String!) {\n        searchSupportedTargets(search: $search)\n      }\n    ":
    types.SearchSupportedTargetsDocument,
  "\n    mutation AssignTarget ($tssPos: Int!, $targetPos: Int!, $targetName: String!) {\n      assignTarget(\n        tssPos: $tssPos\n        targetPos: $targetPos\n        targetName: $targetName\n      ) {\n        tssPos\n        targetPos\n        targetName\n      }\n    }\n  ":
    types.AssignTargetDocument,
  "\n        query AssignedTargets{\n          assignedTargets {\n            state\n            data {\n              name\n              flashStatus\n              flashMessage\n            }\n          }\n        }\n      ":
    types.AssignedTargetsDocument,
  "\nquery AvailableTestPrograms{\n  availableTestprograms {\n    name\n    testprogramArm {\n      architecture\n      status\n      compileMessage\n    }\n    testprogramRiscv {\n      architecture\n      status\n      compileMessage\n    }\n  }\n}\n":
    types.AvailableTestProgramsDocument,
  "\n    query TestProgram ($testprogramName: String!) {\n      testprogram(testprogramName: $testprogramName) {\n        testprogram {\n          name\n          testprogramArm {\n            architecture\n            status\n            compileMessage\n          }\n          testprogramRiscv {\n            architecture\n            status\n            compileMessage\n          }\n        }\n        codeArm\n        codeRiscv\n      }\n    }\n  ":
    types.TestProgramDocument,
  "\n    mutation DeleteTestProgram ($testprogramName: String!) {\n      deleteTestprogram(testprogramName: $testprogramName)\n    }\n  ":
    types.DeleteTestProgramDocument,
  "\n    mutation ModifyTestProgram ($testprogramName: String!, $codeFiles: [Upload!]!) {\n      modifyTestprogram(\n        testprogramName: $testprogramName\n        codeFiles: $codeFiles\n      ) {\n        name\n        testprogramArm {\n          architecture\n          status\n          compileMessage\n        }\n        testprogramRiscv {\n          architecture\n          status\n          compileMessage\n        }\n      }\n    }\n  ":
    types.ModifyTestProgramDocument,
  "\n  query AssignedTargets {\n    assignedTargets {\n      state\n      data {\n        name\n        flashStatus\n        flashMessage\n      }\n    }\n  }\n":
    types.AssignedTargetsDocument,
  "\n    mutation ReinitializeHardware{\n      reinitializeHardware\n    }\n  ":
    types.ReinitializeHardwareDocument,
  "\n        query RegisteredUsers {\n          registeredUsers {\n            username\n            role\n          }\n        }\n      ":
    types.RegisteredUsersDocument,
  "\n    mutation ModifyUser (\n      $oldUsername: String!\n      $newUsername: String\n      $newPassword: String\n      $newRole: Role\n    ) {\n      modifyUser(\n        oldUsername: $oldUsername\n        newUsername: $newUsername\n        newPassword: $newPassword\n        newRole: $newRole\n      ) {\n        username\n        role\n      }\n    }\n  ":
    types.ModifyUserDocument,
  "\n    mutation DeleteUser ($username: String!) {\n      deleteUser(username: $username) {\n        username\n        role\n      }\n    }\n  ":
    types.DeleteUserDocument,
  "\n  query ConnectedTssAndTargets{\n    connectedTss\n    assignedTargets {\n      state\n    }\n  }\n":
    types.ConnectedTssAndTargetsDocument,
  "\n  query TestApiTokens {\n    testApiTokens {\n      name\n      description\n      expiration\n    }\n  }\n":
    types.TestApiTokensDocument,
  "\n    mutation CreateTestApiToken ($name: String!, $description: String!, $expiration: String) {\n      createTestApiToken(\n        name: $name\n        description: $description\n        expiration: $expiration\n      )\n    }\n  ":
    types.CreateTestApiTokenDocument,
  "\n    query ApplicationLog ($application: Application!, $level: LogLevel!) {\n      applicationLog(application: $application, level: $level)\n    }\n  ":
    types.ApplicationLogDocument,
  "\n  query AvailableAndActiveTestPrograms {\n    availableTestprograms {\n      name\n    }\n    activeTestprogram\n  }\n":
    types.AvailableAndActiveTestProgramsDocument,
  "\n    mutation SetActiveTestProgram ($testprogramName: String!) {\n      setActiveTestprogram(testprogramName: $testprogramName)\n    }\n  ":
    types.SetActiveTestProgramDocument,
  "\n        query ActiveTestProgram {\n          activeTestprogram\n        }\n      ":
    types.ActiveTestProgramDocument,
  "\n    mutation CreateTestProgram ($testprogramName: String!) {\n      createTestprogram(testprogramName: $testprogramName) {\n        name\n        testprogramArm {\n          architecture\n          status\n          compileMessage\n        }\n        testprogramRiscv {\n          architecture\n          status\n          compileMessage\n        }\n      }\n    }\n  ":
    types.CreateTestProgramDocument,
  "\n        query AvailableTestprogramsOverview {\n          availableTestprograms {\n            name\n            testprogramArm {\n              architecture\n              status\n              compileMessage\n            }\n            testprogramRiscv {\n              architecture\n              status\n              compileMessage\n            }\n          }\n        }\n      ":
    types.AvailableTestprogramsOverviewDocument,
  "\n  query RegisteredUsers {\n    registeredUsers {\n      username\n      role\n    }\n  }\n":
    types.RegisteredUsersDocument,
  "\n    mutation CreateUser ($username: String!, $password: String!, $role: Role!) {\n      createUser(username: $username, password: $password, role: $role) {\n        username\n        role\n      }\n    }\n  ":
    types.CreateUserDocument,
};

/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = gql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function gql(source: string): unknown;

/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation ChangeUserName ($username: String!) {\n      changeUsername(username: $username) {\n        username\n        role\n      }\n    }\n  ",
): (typeof documents)["\n    mutation ChangeUserName ($username: String!) {\n      changeUsername(username: $username) {\n        username\n        role\n      }\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n        query RegisteredUsers{\n          registeredUsers {\n            username\n            role\n          }\n        }\n      ",
): (typeof documents)["\n        query RegisteredUsers{\n          registeredUsers {\n            username\n            role\n          }\n        }\n      "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation ChangePassword ($oldPassword: String!, $newPassword: String!) {\n      changePassword(oldPassword: $oldPassword, newPassword: $newPassword)\n    }\n  ",
): (typeof documents)["\n    mutation ChangePassword ($oldPassword: String!, $newPassword: String!) {\n      changePassword(oldPassword: $oldPassword, newPassword: $newPassword)\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation RevokeTestApiToken ($name: String!) {\n      revokeTestApiToken(name: $name)\n    }\n  ",
): (typeof documents)["\n    mutation RevokeTestApiToken ($name: String!) {\n      revokeTestApiToken(name: $name)\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n        query TestApiTokens {\n          testApiTokens {\n            name\n            description\n            expiration\n          }\n        }\n      ",
): (typeof documents)["\n        query TestApiTokens {\n          testApiTokens {\n            name\n            description\n            expiration\n          }\n        }\n      "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  query AssignedAndConnectedProbes {\n    assignedProbes {\n      state\n      data {\n        identifier\n        serialNumber\n      }\n    }\n    connectedProbes {\n      identifier\n      serialNumber\n    }\n  }\n",
): (typeof documents)["\n  query AssignedAndConnectedProbes {\n    assignedProbes {\n      state\n      data {\n        identifier\n        serialNumber\n      }\n    }\n    connectedProbes {\n      identifier\n      serialNumber\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation AssignProbe ($probePos: Int!, $probeState: FlatProbeStateInput!) {\n      assignProbe(probePos: $probePos, probeState: $probeState) {\n        probePos\n        data {\n          state\n          data {\n            identifier\n            serialNumber\n          }\n        }\n      }\n    }\n  ",
): (typeof documents)["\n    mutation AssignProbe ($probePos: Int!, $probeState: FlatProbeStateInput!) {\n      assignProbe(probePos: $probePos, probeState: $probeState) {\n        probePos\n        data {\n          state\n          data {\n            identifier\n            serialNumber\n          }\n        }\n      }\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n        query AssignedProbesOverview {\n          assignedProbes {\n            state\n            data {\n              identifier\n              serialNumber\n            }\n          }\n        }\n      ",
): (typeof documents)["\n        query AssignedProbesOverview {\n          assignedProbes {\n            state\n            data {\n              identifier\n              serialNumber\n            }\n          }\n        }\n      "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    query AssignedProbes {\n      assignedProbes {\n        state\n        data {\n          identifier\n          serialNumber\n        }\n      }\n      connectedProbes {\n        identifier\n        serialNumber\n      }\n    }\n  ",
): (typeof documents)["\n    query AssignedProbes {\n      assignedProbes {\n        state\n        data {\n          identifier\n          serialNumber\n        }\n      }\n      connectedProbes {\n        identifier\n        serialNumber\n      }\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation ReinitializeHardware {\n      reinitializeHardware\n    }\n  ",
): (typeof documents)["\n    mutation ReinitializeHardware {\n      reinitializeHardware\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    query SystemInfo {\n      systemInfo {\n        controller\n        soc\n        hostname\n        cores\n        os\n        memory {\n          total\n          free\n        }\n        disk {\n          total\n          free\n        }\n        averageLoad\n      }\n    }\n  ",
): (typeof documents)["\n    query SystemInfo {\n      systemInfo {\n        controller\n        soc\n        hostname\n        cores\n        os\n        memory {\n          total\n          free\n        }\n        disk {\n          total\n          free\n        }\n        averageLoad\n      }\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n      query SearchSupportedTargets ($search: String!) {\n        searchSupportedTargets(search: $search)\n      }\n    ",
): (typeof documents)["\n      query SearchSupportedTargets ($search: String!) {\n        searchSupportedTargets(search: $search)\n      }\n    "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation AssignTarget ($tssPos: Int!, $targetPos: Int!, $targetName: String!) {\n      assignTarget(\n        tssPos: $tssPos\n        targetPos: $targetPos\n        targetName: $targetName\n      ) {\n        tssPos\n        targetPos\n        targetName\n      }\n    }\n  ",
): (typeof documents)["\n    mutation AssignTarget ($tssPos: Int!, $targetPos: Int!, $targetName: String!) {\n      assignTarget(\n        tssPos: $tssPos\n        targetPos: $targetPos\n        targetName: $targetName\n      ) {\n        tssPos\n        targetPos\n        targetName\n      }\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n        query AssignedTargets{\n          assignedTargets {\n            state\n            data {\n              name\n              flashStatus\n              flashMessage\n            }\n          }\n        }\n      ",
): (typeof documents)["\n        query AssignedTargets{\n          assignedTargets {\n            state\n            data {\n              name\n              flashStatus\n              flashMessage\n            }\n          }\n        }\n      "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\nquery AvailableTestPrograms{\n  availableTestprograms {\n    name\n    testprogramArm {\n      architecture\n      status\n      compileMessage\n    }\n    testprogramRiscv {\n      architecture\n      status\n      compileMessage\n    }\n  }\n}\n",
): (typeof documents)["\nquery AvailableTestPrograms{\n  availableTestprograms {\n    name\n    testprogramArm {\n      architecture\n      status\n      compileMessage\n    }\n    testprogramRiscv {\n      architecture\n      status\n      compileMessage\n    }\n  }\n}\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    query TestProgram ($testprogramName: String!) {\n      testprogram(testprogramName: $testprogramName) {\n        testprogram {\n          name\n          testprogramArm {\n            architecture\n            status\n            compileMessage\n          }\n          testprogramRiscv {\n            architecture\n            status\n            compileMessage\n          }\n        }\n        codeArm\n        codeRiscv\n      }\n    }\n  ",
): (typeof documents)["\n    query TestProgram ($testprogramName: String!) {\n      testprogram(testprogramName: $testprogramName) {\n        testprogram {\n          name\n          testprogramArm {\n            architecture\n            status\n            compileMessage\n          }\n          testprogramRiscv {\n            architecture\n            status\n            compileMessage\n          }\n        }\n        codeArm\n        codeRiscv\n      }\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation DeleteTestProgram ($testprogramName: String!) {\n      deleteTestprogram(testprogramName: $testprogramName)\n    }\n  ",
): (typeof documents)["\n    mutation DeleteTestProgram ($testprogramName: String!) {\n      deleteTestprogram(testprogramName: $testprogramName)\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation ModifyTestProgram ($testprogramName: String!, $codeFiles: [Upload!]!) {\n      modifyTestprogram(\n        testprogramName: $testprogramName\n        codeFiles: $codeFiles\n      ) {\n        name\n        testprogramArm {\n          architecture\n          status\n          compileMessage\n        }\n        testprogramRiscv {\n          architecture\n          status\n          compileMessage\n        }\n      }\n    }\n  ",
): (typeof documents)["\n    mutation ModifyTestProgram ($testprogramName: String!, $codeFiles: [Upload!]!) {\n      modifyTestprogram(\n        testprogramName: $testprogramName\n        codeFiles: $codeFiles\n      ) {\n        name\n        testprogramArm {\n          architecture\n          status\n          compileMessage\n        }\n        testprogramRiscv {\n          architecture\n          status\n          compileMessage\n        }\n      }\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  query AssignedTargets {\n    assignedTargets {\n      state\n      data {\n        name\n        flashStatus\n        flashMessage\n      }\n    }\n  }\n",
): (typeof documents)["\n  query AssignedTargets {\n    assignedTargets {\n      state\n      data {\n        name\n        flashStatus\n        flashMessage\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation ReinitializeHardware{\n      reinitializeHardware\n    }\n  ",
): (typeof documents)["\n    mutation ReinitializeHardware{\n      reinitializeHardware\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n        query RegisteredUsers {\n          registeredUsers {\n            username\n            role\n          }\n        }\n      ",
): (typeof documents)["\n        query RegisteredUsers {\n          registeredUsers {\n            username\n            role\n          }\n        }\n      "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation ModifyUser (\n      $oldUsername: String!\n      $newUsername: String\n      $newPassword: String\n      $newRole: Role\n    ) {\n      modifyUser(\n        oldUsername: $oldUsername\n        newUsername: $newUsername\n        newPassword: $newPassword\n        newRole: $newRole\n      ) {\n        username\n        role\n      }\n    }\n  ",
): (typeof documents)["\n    mutation ModifyUser (\n      $oldUsername: String!\n      $newUsername: String\n      $newPassword: String\n      $newRole: Role\n    ) {\n      modifyUser(\n        oldUsername: $oldUsername\n        newUsername: $newUsername\n        newPassword: $newPassword\n        newRole: $newRole\n      ) {\n        username\n        role\n      }\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation DeleteUser ($username: String!) {\n      deleteUser(username: $username) {\n        username\n        role\n      }\n    }\n  ",
): (typeof documents)["\n    mutation DeleteUser ($username: String!) {\n      deleteUser(username: $username) {\n        username\n        role\n      }\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  query ConnectedTssAndTargets{\n    connectedTss\n    assignedTargets {\n      state\n    }\n  }\n",
): (typeof documents)["\n  query ConnectedTssAndTargets{\n    connectedTss\n    assignedTargets {\n      state\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  query TestApiTokens {\n    testApiTokens {\n      name\n      description\n      expiration\n    }\n  }\n",
): (typeof documents)["\n  query TestApiTokens {\n    testApiTokens {\n      name\n      description\n      expiration\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation CreateTestApiToken ($name: String!, $description: String!, $expiration: String) {\n      createTestApiToken(\n        name: $name\n        description: $description\n        expiration: $expiration\n      )\n    }\n  ",
): (typeof documents)["\n    mutation CreateTestApiToken ($name: String!, $description: String!, $expiration: String) {\n      createTestApiToken(\n        name: $name\n        description: $description\n        expiration: $expiration\n      )\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    query ApplicationLog ($application: Application!, $level: LogLevel!) {\n      applicationLog(application: $application, level: $level)\n    }\n  ",
): (typeof documents)["\n    query ApplicationLog ($application: Application!, $level: LogLevel!) {\n      applicationLog(application: $application, level: $level)\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  query AvailableAndActiveTestPrograms {\n    availableTestprograms {\n      name\n    }\n    activeTestprogram\n  }\n",
): (typeof documents)["\n  query AvailableAndActiveTestPrograms {\n    availableTestprograms {\n      name\n    }\n    activeTestprogram\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation SetActiveTestProgram ($testprogramName: String!) {\n      setActiveTestprogram(testprogramName: $testprogramName)\n    }\n  ",
): (typeof documents)["\n    mutation SetActiveTestProgram ($testprogramName: String!) {\n      setActiveTestprogram(testprogramName: $testprogramName)\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n        query ActiveTestProgram {\n          activeTestprogram\n        }\n      ",
): (typeof documents)["\n        query ActiveTestProgram {\n          activeTestprogram\n        }\n      "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation CreateTestProgram ($testprogramName: String!) {\n      createTestprogram(testprogramName: $testprogramName) {\n        name\n        testprogramArm {\n          architecture\n          status\n          compileMessage\n        }\n        testprogramRiscv {\n          architecture\n          status\n          compileMessage\n        }\n      }\n    }\n  ",
): (typeof documents)["\n    mutation CreateTestProgram ($testprogramName: String!) {\n      createTestprogram(testprogramName: $testprogramName) {\n        name\n        testprogramArm {\n          architecture\n          status\n          compileMessage\n        }\n        testprogramRiscv {\n          architecture\n          status\n          compileMessage\n        }\n      }\n    }\n  "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n        query AvailableTestprogramsOverview {\n          availableTestprograms {\n            name\n            testprogramArm {\n              architecture\n              status\n              compileMessage\n            }\n            testprogramRiscv {\n              architecture\n              status\n              compileMessage\n            }\n          }\n        }\n      ",
): (typeof documents)["\n        query AvailableTestprogramsOverview {\n          availableTestprograms {\n            name\n            testprogramArm {\n              architecture\n              status\n              compileMessage\n            }\n            testprogramRiscv {\n              architecture\n              status\n              compileMessage\n            }\n          }\n        }\n      "];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  query RegisteredUsers {\n    registeredUsers {\n      username\n      role\n    }\n  }\n",
): (typeof documents)["\n  query RegisteredUsers {\n    registeredUsers {\n      username\n      role\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation CreateUser ($username: String!, $password: String!, $role: Role!) {\n      createUser(username: $username, password: $password, role: $role) {\n        username\n        role\n      }\n    }\n  ",
): (typeof documents)["\n    mutation CreateUser ($username: String!, $password: String!, $role: Role!) {\n      createUser(username: $username, password: $password, role: $role) {\n        username\n        role\n      }\n    }\n  "];

export function gql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> =
  TDocumentNode extends DocumentNode<infer TType, any> ? TType : never;
