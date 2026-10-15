// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { isIPv4 } from 'node:net';
import { MxcError, run, type ContainerRequest } from '@microsoft/mxc-sdk/v1';

try {
  const ip = process.env.MXC_PROXY_IP ?? '127.0.0.1';
  if (!isIPv4(ip)) {
    throw new Error('MXC_PROXY_IP must be an IPv4 address.');
  }
  const loopback = ip.startsWith('127.');
  const proxy = `http://${ip}:8080`;
  const request: ContainerRequest = {
    command: 'curl.exe --fail --show-error --silent --max-time 20 --noproxy "" https://www.example.com/',
    containment: { type: 'processcontainer' },
    network: {
      egress: {
        default: 'deny',
        allow: [{
          to: [{ cidr: `${ip}/32` }],
          ports: [{ protocol: 'tcp', port: 8080 }],
        }],
      },
      ingress: {
        default: loopback ? 'allow' : 'deny',
        hostLoopback: loopback ? 'allow' : 'deny',
      },
    },
    environment: {
      HTTP_PROXY: proxy,
      HTTPS_PROXY: proxy,
      http_proxy: proxy,
      https_proxy: proxy,
    },
    inheritDefaultEnvironment: true,
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
