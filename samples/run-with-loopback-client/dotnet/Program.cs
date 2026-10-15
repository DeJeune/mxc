// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

using Microsoft.Mxc.Sdk;
using Microsoft.Mxc.Sdk.V1;

try
{
    var request = new ContainerRequest(
        "curl.exe --fail --show-error --silent --max-time 10 --proxy \"\" http://127.0.0.1:8081/")
    {
        Containment = new Containment.ProcessContainer(),
        Network = new NetworkPolicy
        {
            Egress = new NetworkEgressPolicy
            {
                Default = NetworkAction.Deny,
                Allow = [new NetworkRulePolicy
                {
                    To = [new NetworkPeerPolicy("127.0.0.1/32")],
                    Ports = [new NetworkPortPolicy { Protocol = NetworkProtocol.Tcp, Port = 8081 }],
                }],
            },
            Ingress = new NetworkIngressPolicy
            {
                Default = NetworkAction.Deny,
                HostLoopback = NetworkAction.Allow,
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
