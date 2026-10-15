// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createRequest } from './requests.js';

test('runtime proxy scopes an identified peer without opening host loopback', () => {
  const request = createRequest('runtime-proxy', { MXC_PROXY_PEER: 'Contoso.Proxy_123' });
  assert.deepEqual(request.containment, {
    type: 'processcontainer',
    config: { network: { allowedProxyPeer: 'Contoso.Proxy_123' } },
  });
  assert.deepEqual(request.network, {
    egress: { default: 'deny' },
    ingress: { default: 'allow', hostLoopback: 'deny' },
    runtimeConfig: { networkProxy: 'http://127.0.0.1:8080' },
  });
  assert.equal(request.environment, undefined);
});

test('identity-less runtime proxy omits the peer and explicitly allows host loopback', () => {
  const request = createRequest('runtime-proxy', {});
  assert.deepEqual(request.containment, { type: 'processcontainer' });
  assert.equal(request.network?.ingress?.hostLoopback, 'allow');
  assert.equal(request.network?.egress?.allow, undefined);
  assert.throws(() => createRequest('runtime-proxy', { MXC_PROXY_PEER: ' ' }), /non-blank/);
});

for (const [ip, loopback] of [['127.0.0.1', true], ['203.0.113.10', false]] as const) {
  test(`cooperative proxy uses direct egress to ${ip} and explicit environment variables`, () => {
    const request = createRequest('cooperative-proxy', { MXC_PROXY_IP: ip });
    assert.equal(request.network?.runtimeConfig, undefined);
    assert.deepEqual(request.network?.egress, {
      default: 'deny',
      allow: [{
        to: [{ cidr: `${ip}/32` }],
        ports: [{ protocol: 'tcp', port: 8080 }],
      }],
    });
    assert.deepEqual(request.network?.ingress, {
      default: loopback ? 'allow' : 'deny',
      hostLoopback: loopback ? 'allow' : 'deny',
    });
    assert.equal(request.environment?.HTTPS_PROXY, `http://${ip}:8080`);
    assert.equal(request.environment?.https_proxy, `http://${ip}:8080`);
    assert.equal(request.inheritDefaultEnvironment, true);
    assert.ok(request.command.includes('--noproxy ""'));
  });
}

test('loopback server uses an encoded script and either requested egress default', () => {
  for (const scenario of ['loopback-server', 'loopback-server-egress-allow']) {
    const request = createRequest(scenario, {});
    assert.equal(request.network?.egress?.default, scenario === 'loopback-server' ? 'deny' : 'allow');
    assert.deepEqual(request.network?.ingress, { default: 'deny', hostLoopback: 'allow' });
    const encoded = request.command.split(' -EncodedCommand ')[1];
    const script = Buffer.from(encoded, 'base64').toString('utf16le');
    assert.ok(script.includes('[Net.IPAddress]::Loopback, 8081'));
    assert.ok(script.includes('[string][char]13 + [char]10'));
    assert.ok(script.includes('$client.Dispose()'));
    assert.ok(script.includes('$listener.Stop()'));
    assert.equal(request.timeoutMs, 60_000);
  }
});

test('loopback client permits only the listener address and TCP port', () => {
  const request = createRequest('loopback-client', {});
  assert.deepEqual(request.network?.egress?.allow, [{
    to: [{ cidr: '127.0.0.1/32' }],
    ports: [{ protocol: 'tcp', port: 8081 }],
  }]);
  assert.deepEqual(request.network?.ingress, { default: 'deny', hostLoopback: 'allow' });
  assert.ok(request.command.includes('--proxy ""'));
});

test('public egress pins the permitted IP while preserving the TLS hostname', () => {
  const request = createRequest('public-egress', {
    MXC_DESTINATION_IP: '203.0.113.10',
    MXC_DESTINATION_HOST: 'www.example.com',
  });
  assert.deepEqual(request.network?.egress?.allow, [{
    to: [{ cidr: '203.0.113.10/32' }],
    ports: [{ protocol: 'tcp', port: 443 }],
  }]);
  assert.deepEqual(request.network?.ingress, { default: 'deny', hostLoopback: 'deny' });
  assert.ok(request.command.includes('--resolve www.example.com:443:203.0.113.10 https://www.example.com/'));
  assert.ok(request.command.includes('--proxy ""'));
});

test('invalid scenarios and destination values fail explicitly', () => {
  assert.throws(() => createRequest('unknown', {}), /Choose/);
  assert.throws(() => createRequest('cooperative-proxy', { MXC_PROXY_IP: 'localhost' }), /IPv4/);
  assert.throws(() => createRequest('public-egress', {}), /MXC_DESTINATION_IP/);
  assert.throws(() => createRequest('public-egress', {
    MXC_DESTINATION_IP: '203.0.113.10',
    MXC_DESTINATION_HOST: 'example.com & echo bad',
  }), /MXC_DESTINATION_HOST/);
});
