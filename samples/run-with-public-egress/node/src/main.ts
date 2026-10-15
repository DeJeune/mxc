// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { isIPv4 } from 'node:net';
import { MxcError, run, type ContainerRequest } from '@microsoft/mxc-sdk/v1';

try {
  const ip = process.env.MXC_DESTINATION_IP;
  const host = process.env.MXC_DESTINATION_HOST;
  if (ip === undefined || !isIPv4(ip)
      || host === undefined || !/^[a-zA-Z0-9.-]+$/.test(host)) {
    throw new Error('Set MXC_DESTINATION_IP to an IPv4 address and MXC_DESTINATION_HOST to its HTTPS hostname.');
  }
  const request: ContainerRequest = {
    command: 'curl.exe --fail --show-error --silent --max-time 20 --proxy "" '
      + `--resolve ${host}:443:${ip} https://${host}/`,
    containment: { type: 'processcontainer' },
    network: {
      egress: {
        default: 'deny',
        allow: [{
          to: [{ cidr: `${ip}/32` }],
          ports: [{ protocol: 'tcp', port: 443 }],
        }],
      },
      ingress: { default: 'deny', hostLoopback: 'deny' },
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
