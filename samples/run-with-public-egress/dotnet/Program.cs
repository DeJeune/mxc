// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

using System.Net;
using System.Net.Sockets;
using Microsoft.Mxc.Sdk;
using Microsoft.Mxc.Sdk.V1;

var ip = Environment.GetEnvironmentVariable("MXC_DESTINATION_IP");
var host = Environment.GetEnvironmentVariable("MXC_DESTINATION_HOST");
if (!IPAddress.TryParse(ip, out var address)
    || address.AddressFamily != AddressFamily.InterNetwork
    || string.IsNullOrEmpty(host)
    || host.Any(c => !char.IsAsciiLetterOrDigit(c) && c != '.' && c != '-'))
{
    Console.Error.WriteLine(
        "Set MXC_DESTINATION_IP to an IPv4 address and MXC_DESTINATION_HOST to its HTTPS hostname.");
    return 1;
}
ip = address.ToString();

try
{
    var request = new ContainerRequest(
        $"curl.exe --fail --show-error --silent --max-time 20 --proxy \"\" --resolve {host}:443:{ip} https://{host}/")
    {
        Containment = new Containment.ProcessContainer(),
        Network = new NetworkPolicy
        {
            Egress = new NetworkEgressPolicy
            {
                Default = NetworkAction.Deny,
                Allow = [new NetworkRulePolicy
                {
                    To = [new NetworkPeerPolicy($"{ip}/32")],
                    Ports = [new NetworkPortPolicy { Protocol = NetworkProtocol.Tcp, Port = 443 }],
                }],
            },
            Ingress = new NetworkIngressPolicy
            {
                Default = NetworkAction.Deny,
                HostLoopback = NetworkAction.Deny,
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
