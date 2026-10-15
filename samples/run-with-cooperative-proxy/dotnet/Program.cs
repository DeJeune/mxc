// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

using System.Net;
using System.Net.Sockets;
using Microsoft.Mxc.Sdk;
using Microsoft.Mxc.Sdk.V1;

var ip = Environment.GetEnvironmentVariable("MXC_PROXY_IP") ?? "127.0.0.1";
if (!IPAddress.TryParse(ip, out var address)
    || address.AddressFamily != AddressFamily.InterNetwork)
{
    Console.Error.WriteLine("MXC_PROXY_IP must be an IPv4 address.");
    return 1;
}
ip = address.ToString();
var loopback = IPAddress.IsLoopback(address);
var proxy = $"http://{ip}:8080";

try
{
    var request = new ContainerRequest(
        "curl.exe --fail --show-error --silent --max-time 20 --noproxy \"\" https://www.example.com/")
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
                    Ports = [new NetworkPortPolicy { Protocol = NetworkProtocol.Tcp, Port = 8080 }],
                }],
            },
            Ingress = new NetworkIngressPolicy
            {
                Default = loopback ? NetworkAction.Allow : NetworkAction.Deny,
                HostLoopback = loopback ? NetworkAction.Allow : NetworkAction.Deny,
            },
        },
        Environment = new()
        {
            ["HTTP_PROXY"] = proxy, ["HTTPS_PROXY"] = proxy,
            ["http_proxy"] = proxy, ["https_proxy"] = proxy,
        },
        InheritDefaultEnvironment = true,
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
