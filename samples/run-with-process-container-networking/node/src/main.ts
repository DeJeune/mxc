// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { MxcError, run, type ContainerRequest } from '@microsoft/mxc-sdk/v1';

try {
  const peer = process.env.MXC_PROXY_PEER;
  const request: ContainerRequest = {
    command: 'curl.exe --fail --show-error --silent --max-time 20 --noproxy "" https://www.example.com/',
    containment: {
      type: 'processcontainer',
      config: { network: { allowedProxyPeer: peer } },
    },
    network: {
      egress: { default: 'deny' },
      ingress: { default: 'allow', hostLoopback: peer === undefined ? 'allow' : 'deny' },
      runtimeConfig: { networkProxy: 'http://127.0.0.1:8080' },
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
  if (error instanceof MxcError) {
    console.error(`MXC error [${error.code}]: ${error.message}`);
  } else {
    console.error(error);
  }
  process.exitCode = 1;
}
