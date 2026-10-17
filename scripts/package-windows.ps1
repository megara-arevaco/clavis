param(
    [string]$Version,
    [string]$MsysPrefix = $(if ($env:MSYS2_PREFIX) { $env:MSYS2_PREFIX } else { 'C:\msys64\ucrt64' })
)

$ErrorActionPreference = 'Stop'
$Repo = Split-Path -Parent $PSScriptRoot
if (-not $Version) {
    $CargoToml = Get-Content (Join-Path $Repo 'Cargo.toml') -Raw
    if ($CargoToml -notmatch '(?m)^version\s*=\s*"([^"]+)"') { throw 'Could not read the Clavis version from Cargo.toml.' }
    $Version = $Matches[1]
}

$TargetExe = Join-Path $Repo 'target\x86_64-pc-windows-gnu\release\clavis.exe'
$RuntimeBin = Join-Path $MsysPrefix 'bin'
$Objdump = Join-Path $RuntimeBin 'objdump.exe'
$PixbufQuery = Join-Path $RuntimeBin 'gdk-pixbuf-query-loaders.exe'
foreach ($Required in @($TargetExe, $Objdump, $PixbufQuery)) {
    if (-not (Test-Path -LiteralPath $Required -PathType Leaf)) { throw "Required build file not found: $Required" }
}

$Dist = Join-Path $Repo 'dist\windows'
$Stage = Join-Path $Dist 'stage'
if (Test-Path -LiteralPath $Stage) { Remove-Item -LiteralPath $Stage -Recurse -Force }
New-Item -ItemType Directory -Path $Stage -Force | Out-Null
Copy-Item -LiteralPath $TargetExe -Destination (Join-Path $Stage 'clavis.exe')

$LoaderSource = Join-Path $MsysPrefix 'lib\gdk-pixbuf-2.0\2.10.0\loaders'
$LoaderStage = Join-Path $Stage 'lib\gdk-pixbuf-2.0\2.10.0\loaders'
if (-not (Test-Path -LiteralPath $LoaderSource -PathType Container)) { throw "GTK image loaders not found: $LoaderSource" }
New-Item -ItemType Directory -Path $LoaderStage -Force | Out-Null
$LoaderFiles = @(Get-ChildItem -LiteralPath $LoaderSource -Filter '*.dll' -File)
if ($LoaderFiles.Count -eq 0) { throw 'GTK image loader directory is empty.' }
foreach ($Loader in $LoaderFiles) { Copy-Item -LiteralPath $Loader.FullName -Destination $LoaderStage }

# Follow each PE import recursively so the installer contains the GTK runtime DLLs
# without copying the entire compiler toolchain.
$Pending = [System.Collections.Generic.Queue[string]]::new()
$Pending.Enqueue((Join-Path $Stage 'clavis.exe'))
foreach ($Loader in Get-ChildItem -LiteralPath $LoaderStage -Filter '*.dll' -File) { $Pending.Enqueue($Loader.FullName) }
$Visited = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
$CopiedDlls = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
$env:PATH = "$RuntimeBin;$env:PATH"
while ($Pending.Count -gt 0) {
    $Image = $Pending.Dequeue()
    if (-not $Visited.Add($Image)) { continue }
    $ImportOutput = & $Objdump -p $Image 2>&1
    if ($LASTEXITCODE -ne 0) { throw "Could not inspect DLL imports in $Image" }
    foreach ($Line in $ImportOutput) {
        if ($Line -notmatch '^\s*DLL Name:\s*(.+?)\s*$') { continue }
        $DllName = $Matches[1]
        if ($DllName -match '^(api-ms-win|ext-ms-win)-' -or (Test-Path -LiteralPath (Join-Path "$env:WINDIR\System32" $DllName))) { continue }
        $RuntimeDll = Get-ChildItem -LiteralPath $RuntimeBin -Filter $DllName -File | Select-Object -First 1
        if (-not $RuntimeDll) { throw "GTK runtime dependency was not found in $RuntimeBin: $DllName (required by $Image)" }
        $StagedDll = Join-Path $Stage $RuntimeDll.Name
        if ($CopiedDlls.Add($RuntimeDll.Name)) { Copy-Item -LiteralPath $RuntimeDll.FullName -Destination $StagedDll }
        $Pending.Enqueue($StagedDll)
    }
}

# GTK discovers these files relative to GTK_DATA_PREFIX at launch.
$DataItems = @(
    @{ Source = Join-Path $MsysPrefix 'share\glib-2.0\schemas'; Relative = 'share\glib-2.0\schemas' },
    @{ Source = Join-Path $MsysPrefix 'share\icons\Adwaita'; Relative = 'share\icons\Adwaita' },
    @{ Source = Join-Path $MsysPrefix 'share\icons\hicolor'; Relative = 'share\icons\hicolor' },
    @{ Source = Join-Path $MsysPrefix 'share\themes\Adwaita'; Relative = 'share\themes\Adwaita' },
    @{ Source = Join-Path $MsysPrefix 'etc\gtk-4.0'; Relative = 'etc\gtk-4.0' }
)
foreach ($Item in $DataItems) {
    if (Test-Path -LiteralPath $Item.Source -PathType Container) {
        $Destination = Join-Path $Stage $Item.Relative
        New-Item -ItemType Directory -Path $Destination -Force | Out-Null
        Copy-Item -Path (Join-Path $Item.Source '*') -Destination $Destination -Recurse -Force
    }
}

$SchemaDir = Join-Path $Stage 'share\glib-2.0\schemas'
if (Test-Path -LiteralPath $SchemaDir -PathType Container) {
    $SchemaCompiler = Join-Path $RuntimeBin 'glib-compile-schemas.exe'
    if (Test-Path -LiteralPath $SchemaCompiler -PathType Leaf) {
        & $SchemaCompiler $SchemaDir
        if ($LASTEXITCODE -ne 0) { throw 'Could not compile GTK runtime schemas.' }
    }
}

$LoaderPaths = @(Get-ChildItem -LiteralPath $LoaderStage -Filter '*.dll' -File | ForEach-Object FullName)
$PixbufOutput = & $PixbufQuery @LoaderPaths 2>&1
if ($LASTEXITCODE -ne 0) { throw 'Could not generate the GTK image loader cache.' }
$PixbufCache = $PixbufOutput -join "`n"
$StageEscaped = $Stage.Replace('\', '\\')
$PixbufCache = $PixbufCache.Replace($StageEscaped, '@CLAVIS_PREFIX@').Replace($Stage, '@CLAVIS_PREFIX@')
$ClavisShare = Join-Path $Stage 'share\clavis'
New-Item -ItemType Directory -Path $ClavisShare -Force | Out-Null
Set-Content -LiteralPath (Join-Path $ClavisShare 'gdk-pixbuf-loaders.cache.template') -Value $PixbufCache -Encoding ascii

$IconDir = Join-Path $Stage 'share\icons\hicolor\256x256\apps'
New-Item -ItemType Directory -Path $IconDir -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $Repo 'packaging\clavis.png') -Destination (Join-Path $IconDir 'clavis.png')
Copy-Item -LiteralPath (Join-Path $Repo 'LICENSE') -Destination (Join-Path $Stage 'LICENSE')
Copy-Item -LiteralPath (Join-Path $Repo 'README.md') -Destination (Join-Path $Stage 'README.md')

$env:CLAVIS_VERSION = $Version
$InnoCompiler = Get-Command 'ISCC.exe' -ErrorAction SilentlyContinue
if ($InnoCompiler) { $InnoCompilerPath = $InnoCompiler.Source }
if (-not $InnoCompiler) {
    $KnownCompiler = 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
    if (Test-Path -LiteralPath $KnownCompiler -PathType Leaf) { $InnoCompilerPath = $KnownCompiler }
}
if (-not $InnoCompilerPath) { throw 'Inno Setup 6 is required. Install it before packaging Clavis for Windows.' }
& $InnoCompilerPath (Join-Path $Repo 'packaging\windows\clavis.iss')
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup failed to build the Clavis installer.' }
Write-Host "Built Windows installer in $Dist"
