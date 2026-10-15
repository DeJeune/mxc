// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

using Microsoft.Mxc.Sdk;
using Microsoft.Mxc.Sdk.V1;

try
{
    var peer = Environment.GetEnvironmentVariable("MXC_PROXY_PEER");
    var request = new ContainerRequest(
        "curl.exe --fail --show-error --silent --max-time 20 --noproxy \"\" https://www.example.com/")
    {
        Containment = new Containment.ProcessContainer
        {
            Network = new ProcessContainerNetworkPolicy { AllowedProxyPeer = peer },
        },
        Network = new NetworkPolicy
        {
            Egress = new NetworkEgressPolicy { Default = NetworkAction.Deny },
            Ingress = new NetworkIngressPolicy
            {
                Default = NetworkAction.Allow,
                HostLoopback = peer is null ? NetworkAction.Allow : NetworkAction.Deny,
            },
            RuntimeConfig = new NetworkRuntimeConfig
            {
                NetworkProxy = "http://127.0.0.1:8080",
            },
        },
        TimeoutMs = 30_000,
    };
    var result = await MxcContainer.RunAsync(request);

    Console.Write(result.Stdout);
    Console.Error.Write(result.Stderr);
    foreach (var warning in result.Warnings)
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
