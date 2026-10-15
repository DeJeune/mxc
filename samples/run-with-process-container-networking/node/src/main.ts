// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { MxcError, probe, spawn } from '@microsoft/mxc-sdk/v1';
import { createRequest } from './requests.js';

async function main(): Promise<number> {
  if (process.platform !== 'win32') {
    throw new Error('These samples require a prepared Windows ProcessContainer host.');
  }
  const request = createRequest(process.argv[2] ?? '');
  const support = probe(request);
  for (const warning of support.warnings) {
    console.error(`probe warning: ${warning}`);
  }
  if (support.tier !== 'base-container') {
    throw new Error(support.error ?? 'This request requires the BaseContainer tier.');
  }

  const containerProcess = await spawn(request);
  try {
    const stdout = containerProcess.standardOutput;
    const stderr = containerProcess.standardError;
    if (stdout === null || stderr === null) {
      throw new Error('ProcessContainer did not provide the expected output streams.');
    }
    const forward = async (
      stream: NonNullable<typeof stdout>,
      destination: NodeJS.WritableStream,
    ): Promise<void> => {
      for await (const chunk of stream) {
        destination.write(chunk);
      }
    };
    const [result] = await Promise.all([
      containerProcess.wait(),
      forward(stdout, process.stdout),
      forward(stderr, process.stderr),
    ]);
    for (const warning of containerProcess.warnings) {
      console.error(`warning: ${warning}`);
    }
    if (result.timedOut) {
      console.error('workload timed out');
      return 124;
    }
    return result.exitCode;
  } finally {
    containerProcess.dispose();
  }
}

try {
  process.exitCode = await main();
} catch (error) {
  if (error instanceof MxcError) {
    console.error(`MXC error [${error.code}]: ${error.message}`);
  } else {
    console.error(error);
  }
  process.exitCode = 1;
}
