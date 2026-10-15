// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

using Microsoft.Mxc.Sdk;
using Microsoft.Mxc.Sdk.V1;

try
{
    var script = Path.Combine(AppContext.BaseDirectory, "server.ps1");
    var request = new ContainerRequest(
        $"powershell.exe -NoLogo -NoProfile -NonInteractive -File \"{script}\"")
    {
        Containment = new Containment.ProcessContainer(),
        Filesystem = new FilesystemPolicy { ReadonlyPaths = { AppContext.BaseDirectory } },
        Network = new NetworkPolicy
        {
            Egress = new NetworkEgressPolicy { Default = NetworkAction.Deny },
            Ingress = new NetworkIngressPolicy
            {
                Default = NetworkAction.Deny,
                HostLoopback = NetworkAction.Allow,
            },
        },
        TimeoutMs = 60_000,
    };
    using var process = await MxcContainer.SpawnAsync(request);
    var stdout = process.StandardOutput ?? throw new IOException("Missing container stdout.");
    var stderr = process.StandardError ?? throw new IOException("Missing container stderr.");
    var output = stdout.CopyToAsync(Console.OpenStandardOutput());
    var errors = stderr.CopyToAsync(Console.OpenStandardError());
    var result = await process.WaitAsync();
    await Task.WhenAll(output, errors);

    foreach (var warning in process.Warnings)
    {
        Console.Error.WriteLine($"warning: {warning}");
    }
    if (result.TimedOut)
    {
        Console.Error.WriteLine("workload timed out");
        return 124;
    }
    return result.ExitCode;
}
catch (MxcException error)
{
    Console.Error.WriteLine($"MXC error [{error.Code}]: {error.Message}");
    return 1;
}
catch (IOException error)
{
    Console.Error.WriteLine($"stream error: {error.Message}");
    return 1;
}
