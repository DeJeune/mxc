// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { isIPv4 } from 'node:net';
import type { ContainerRequest } from '@microsoft/mxc-sdk/v1';

const proxyCommand =
  'curl.exe --fail --show-error --silent --max-time 20 --noproxy "" https://www.example.com/';

const serverScript = `
$ErrorActionPreference = 'Stop'
$listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 8081)
$client = $null
try {
  $listener.Start()
  Write-Output 'LISTENING=http://127.0.0.1:8081/'
  $client = $listener.AcceptTcpClient()
  $client.ReceiveTimeout = 5000
  $client.SendTimeout = 5000
  $stream = $client.GetStream()
  $buffer = [byte[]]::new(4096)
  $null = $stream.Read($buffer, 0, $buffer.Length)
  $crlf = [string][char]13 + [char]10
  $response = [Text.Encoding]::ASCII.GetBytes(
    (@('HTTP/1.1 200 OK', 'Content-Length: 2', 'Connection: close', '', 'OK') -join $crlf))
  $stream.Write($response, 0, $response.Length)
  $stream.Flush()
} finally {
  if ($null -ne $client) { $client.Dispose() }
  $listener.Stop()
}
`;

export function createRequest(
  scenario: string,
  environment: NodeJS.ProcessEnv = process.env,
): ContainerRequest {
  const request: ContainerRequest = {
    containment: { type: 'processcontainer' },
    command: proxyCommand,
    timeoutMs: 30_000,
  };

  switch (scenario) {
    case 'runtime-proxy': {
      const peer = environment.MXC_PROXY_PEER;
      if (peer !== undefined && peer.trim().length === 0) {
        throw new Error('MXC_PROXY_PEER must be non-blank, or unset for an identity-less proxy.');
      }
      request.containment = {
        type: 'processcontainer',
        ...(peer === undefined ? {} : { config: { network: { allowedProxyPeer: peer } } }),
      };
      request.network = {
        egress: { default: 'deny' },
        ingress: { default: 'allow', hostLoopback: peer === undefined ? 'allow' : 'deny' },
        runtimeConfig: { networkProxy: 'http://127.0.0.1:8080' },
      };
      break;
    }
    case 'cooperative-proxy': {
      const proxyIp = environment.MXC_PROXY_IP ?? '127.0.0.1';
      if (!isIPv4(proxyIp)) {
        throw new Error('MXC_PROXY_IP must be an IPv4 address, not a hostname or URL.');
      }
      const loopback = proxyIp.startsWith('127.');
      const proxyUrl = `http://${proxyIp}:8080`;
      request.network = {
        egress: {
          default: 'deny',
          allow: [{
            to: [{ cidr: `${proxyIp}/32` }],
            ports: [{ protocol: 'tcp', port: 8080 }],
          }],
        },
        ingress: {
          default: loopback ? 'allow' : 'deny',
          hostLoopback: loopback ? 'allow' : 'deny',
        },
      };
      request.environment = {
        HTTP_PROXY: proxyUrl,
        HTTPS_PROXY: proxyUrl,
        http_proxy: proxyUrl,
        https_proxy: proxyUrl,
      };
      request.inheritDefaultEnvironment = true;
      break;
    }
    case 'loopback-server':
    case 'loopback-server-egress-allow':
      request.command = 'powershell.exe -NoLogo -NoProfile -NonInteractive -EncodedCommand '
        + Buffer.from(serverScript, 'utf16le').toString('base64');
      request.timeoutMs = 60_000;
      request.network = {
        egress: { default: scenario === 'loopback-server' ? 'deny' : 'allow' },
        ingress: { default: 'deny', hostLoopback: 'allow' },
      };
      break;
    case 'loopback-client':
      request.command =
        'curl.exe --fail --show-error --silent --max-time 10 --proxy "" http://127.0.0.1:8081/';
      request.network = {
        egress: {
          default: 'deny',
          allow: [{
            to: [{ cidr: '127.0.0.1/32' }],
            ports: [{ protocol: 'tcp', port: 8081 }],
          }],
        },
        ingress: { default: 'deny', hostLoopback: 'allow' },
      };
      break;
    case 'public-egress': {
      const ip = environment.MXC_DESTINATION_IP;
      const host = environment.MXC_DESTINATION_HOST;
      if (ip === undefined || !isIPv4(ip)) {
        throw new Error('Set MXC_DESTINATION_IP to the permitted public IPv4 address.');
      }
      if (host === undefined || !/^[a-zA-Z0-9.-]+$/.test(host)) {
        throw new Error('Set MXC_DESTINATION_HOST to the HTTPS server hostname.');
      }
      request.command = 'curl.exe --fail --show-error --silent --max-time 20 --proxy "" '
        + `--resolve ${host}:443:${ip} https://${host}/`;
      request.network = {
        egress: {
          default: 'deny',
          allow: [{
            to: [{ cidr: `${ip}/32` }],
            ports: [{ protocol: 'tcp', port: 443 }],
          }],
        },
        ingress: { default: 'deny', hostLoopback: 'deny' },
      };
      break;
    }
    default:
      throw new Error(
        'Choose runtime-proxy, cooperative-proxy, loopback-server, '
        + 'loopback-server-egress-allow, loopback-client, or public-egress.',
      );
  }

  return request;
}
