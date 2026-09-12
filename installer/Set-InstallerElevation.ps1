param(
    [Parameter(Mandatory)][string]$Installer,
    [Parameter(Mandatory)][string]$Application
)
$ErrorActionPreference = 'Stop'
$Installer = (Resolve-Path $Installer).Path
$Application = (Resolve-Path $Application).Path
$mt = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\mt.exe" |
    Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
if (!$mt) { throw 'Windows SDK manifest tool (mt.exe) was not found' }
$temporary = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
New-Item -ItemType Directory $temporary | Out-Null

function Read-Manifest([string]$Executable, [string]$Name) {
    $path = Join-Path $temporary $Name
    & $mt -nologo "-inputresource:$Executable;#1" "-out:$path"
    if ($LASTEXITCODE -ne 0) { throw "Failed to extract manifest: $Executable" }
    $document = New-Object System.Xml.XmlDocument
    $document.PreserveWhitespace = $true
    $document.Load($path)
    return ,$document
}

function Execution-Level($Document) {
    $nodes = $Document.SelectNodes("//*[local-name()='requestedExecutionLevel']")
    if ($nodes.Count -ne 1) { throw 'Expected exactly one requestedExecutionLevel' }
    return $nodes[0]
}

try {
    $applicationHash = (Get-FileHash $Application).Hash
    $appManifest = Read-Manifest $Application 'application.xml'
    if ((Execution-Level $appManifest).GetAttribute('level') -ne 'asInvoker') {
        throw 'The application must remain asInvoker'
    }
    $manifest = Read-Manifest $Installer 'installer.xml'
    $level = Execution-Level $manifest
    $level.SetAttribute('level', 'requireAdministrator')
    $level.SetAttribute('uiAccess', 'false')
    $path = Join-Path $temporary 'elevated.xml'
    $manifest.Save($path)
    # Run after ISCC and before signing. Preserve the loader's other manifest settings.
    & $mt -nologo -manifest $path "-outputresource:$Installer;#1"
    if ($LASTEXITCODE -ne 0) { throw 'Failed to update installer manifest' }
    $verified = Read-Manifest $Installer 'verified.xml'
    if ((Execution-Level $verified).GetAttribute('level') -ne 'requireAdministrator') {
        throw 'Installer must request elevation before any loader code runs'
    }
    if ((Get-FileHash $Application).Hash -ne $applicationHash) {
        throw 'Application executable unexpectedly changed'
    }
    Write-Host 'Verified: installer=requireAdministrator; application=asInvoker (unchanged).'
} finally {
    Remove-Item $temporary -Recurse -Force
}
