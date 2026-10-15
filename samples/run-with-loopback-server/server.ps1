# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

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
    $response = [Text.Encoding]::ASCII.GetBytes(
        "HTTP/1.1 200 OK`r`nContent-Length: 2`r`nConnection: close`r`n`r`nOK")
    $stream.Write($response, 0, $response.Length)
    $stream.Flush()
} finally {
    if ($null -ne $client) { $client.Dispose() }
    $listener.Stop()
}
