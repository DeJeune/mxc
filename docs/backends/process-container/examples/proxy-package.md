# Packaged AppContainer proxy

> **Audience:** MXC consumers and developers

The [runtime proxy code sample](../../../../samples/run-with-process-container-networking/README.md#runtime-proxy)
can connect to a packaged proxy using its Package Family Name. The package
must provide its own proxy executable, signing, and assets. The manifest below
shows the application identity, AppContainer capabilities, and inbound
firewall declaration for a proxy listening on port 8080.

## Manifest

```xml
<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:uap10="http://schemas.microsoft.com/appx/manifest/uap/windows10/10"
  xmlns:desktop2="http://schemas.microsoft.com/appx/manifest/desktop/windows10/2"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  IgnorableNamespaces="uap uap10 desktop2 rescap">
  <Identity Name="Contoso.AgentProxy" Publisher="CN=Contoso" Version="1.0.0.0" ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>Agent Proxy</DisplayName>
    <PublisherDisplayName>Contoso</PublisherDisplayName>
    <Logo>Assets\Logo.png</Logo>
  </Properties>
  <Resources>
    <Resource Language="en-us" />
  </Resources>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.19041.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Capabilities>
    <Capability Name="internetClient" />
    <Capability Name="privateNetworkClientServer" />
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
  <Extensions>
    <desktop2:Extension Category="windows.firewallRules">
      <desktop2:FirewallRules Executable="proxy.exe">
        <desktop2:Rule Direction="in" IPProtocol="TCP" Profile="all"
          LocalPortMin="8080" LocalPortMax="8080" />
      </desktop2:FirewallRules>
    </desktop2:Extension>
  </Extensions>
  <Applications>
    <Application Id="Proxy" Executable="proxy.exe"
      uap10:RuntimeBehavior="packagedClassicApp" uap10:TrustLevel="appContainer">
      <uap:VisualElements DisplayName="Agent Proxy"
        Description="HTTP proxy for contained workloads" BackgroundColor="transparent"
        Square150x150Logo="Assets\Logo.png" Square44x44Logo="Assets\Logo.png"
        AppListEntry="none" />
    </Application>
  </Applications>
</Package>
```

See the Microsoft Learn
[package manifest schema reference](https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/schema-root)
for package identity, signing, assets, and application metadata.
`runFullTrust` is required by the firewall extension;
`TrustLevel="appContainer"` still runs the proxy in an AppContainer.

The manifest example uses port 8080. The proxy developer must ensure that its inbound firewall rule covers the port
supplied to MXC in `runtimeConfig.networkProxy`.

After installing the package, retrieve its exact Package Family Name:

```powershell
Get-AppxPackage -Name Contoso.AgentProxy |
  Select-Object -ExpandProperty PackageFamilyName
```

Use that value verbatim for `allowedProxyPeer` for any packaged proxy. For an unpackaged AppContainer proxy, use its
AppContainer profile name instead.

See [Proxy identity and firewall authorization](../networking.md#proxy-identity-and-firewall-authorization)
for Windows enforcement and firewall details.
