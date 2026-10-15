// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { finished } from 'node:stream/promises';
import { MxcError, spawn, type ContainerRequest } from '@microsoft/mxc-sdk/v1';

try {
  const script = fileURLToPath(new URL('../../server.ps1', import.meta.url));
  const request: ContainerRequest = {
    command: `powershell.exe -NoLogo -NoProfile -NonInteractive -File "${script}"`,
    containment: { type: 'processcontainer' },
    filesystem: { readonlyPaths: [path.dirname(script)] },
    network: {
      egress: { default: 'deny' },
      ingress: { default: 'deny', hostLoopback: 'allow' },
    },
    timeoutMs: 60_000,
  };
  const containerProcess = await spawn(request);
  try {
    const stdout = containerProcess.standardOutput;
    const stderr = containerProcess.standardError;
    if (stdout === null || stderr === null) {
      throw new Error('ProcessContainer did not provide the expected output streams.');
    }
    stdout.pipe(process.stdout, { end: false });
    stderr.pipe(process.stderr, { end: false });
    const [result] = await Promise.all([
      containerProcess.wait(),
      finished(stdout),
      finished(stderr),
    ]);
    for (const warning of containerProcess.warnings) {
      console.error(`warning: ${warning}`);
    }
    if (result.timedOut) {
      console.error('workload timed out');
    }
    process.exitCode = result.timedOut ? 124 : result.exitCode;
  } finally {
    containerProcess.dispose();
  }
} catch (error) {
  console.error(error instanceof MxcError
    ? `MXC error [${error.code}]: ${error.message}`
    : error);
  process.exitCode = 1;
}
