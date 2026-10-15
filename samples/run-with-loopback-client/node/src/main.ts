// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { MxcError, run, type ContainerRequest } from '@microsoft/mxc-sdk/v1';

try {
  const request: ContainerRequest = {
    command: 'curl.exe --fail --show-error --silent --max-time 10 --proxy "" http://127.0.0.1:8081/',
    containment: { type: 'processcontainer' },
    network: {
      egress: {
        default: 'deny',
        allow: [{
          to: [{ cidr: '127.0.0.1/32' }],
          ports: [{ protocol: 'tcp', port: 8081 }],
        }],
      },
      ingress: { default: 'deny', hostLoopback: 'allow' },
    },
    timeoutMs: 30_000,
  };
  const result = await run(request);

  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
  for (const warning of result.warnings) {
    console.error(`warning: ${warning}`);
  }
  if (result.timedOut) {
    console.error('workload timed out');
  }
  process.exitCode = result.timedOut ? 124 : result.exitCode;
} catch (error) {
  console.error(error instanceof MxcError
    ? `MXC error [${error.code}]: ${error.message}`
    : error);
  process.exitCode = 1;
}
