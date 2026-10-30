# Functional acceptance test for an installed or extracted domainforge.exe.
#
# Usage: packaging/windows/smoke.ps1 <domainforge.exe> <valid.sea> <invalid.sea> <version>
param(
    [Parameter(Mandatory = $true, Position = 0)][string]$Cli,
    [Parameter(Mandatory = $true, Position = 1)][string]$Valid,
    [Parameter(Mandatory = $true, Position = 2)][string]$Invalid,
    [Parameter(Mandatory = $true, Position = 3)][string]$ExpectedVersion
)

$ErrorActionPreference = 'Stop'

function Fail([string]$Message) {
    Write-Error "FAIL: $Message"
    exit 1
}

if (-not (Test-Path $Cli)) { Fail "not found: $Cli" }
if (-not (Test-Path $Valid)) { Fail "fixture not found: $Valid" }
if (-not (Test-Path $Invalid)) { Fail "fixture not found: $Invalid" }

$python = Get-Command python -ErrorAction SilentlyContinue
if (-not $python) { $python = Get-Command python3 -ErrorAction SilentlyContinue }
if (-not $python) { Fail 'python is required for the JSON and projection checks' }

$work = Join-Path ([System.IO.Path]::GetTempPath()) ("domainforge-smoke-" + [guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $work | Out-Null

function Get-DirHashes([string]$Dir) {
    Get-ChildItem -Path $Dir -Recurse -File | ForEach-Object {
        $rel = $_.FullName.Substring($Dir.Length).TrimStart('\', '/')
        "$rel=$((Get-FileHash -Path $_.FullName -Algorithm SHA256).Hash)"
    } | Sort-Object
}

try {
    $version = (& $Cli --version | Out-String).Trim()
    if ($version -ne "domainforge $ExpectedVersion") {
        Fail "version output '$version' != 'domainforge $ExpectedVersion'"
    }
    Write-Host "smoke: version -> $version"

    & $Cli validate --format human --no-color $Valid *> (Join-Path $work 'validate.out')
    if ($LASTEXITCODE -ne 0) { Fail 'valid model was rejected' }
    if (-not (Select-String -Path (Join-Path $work 'validate.out') -Pattern 'Validation succeeded' -Quiet)) {
        Fail 'valid model output did not report success'
    }
    Write-Host 'smoke: valid model accepted'

    & $Cli validate --format human --no-color $Invalid *> (Join-Path $work 'invalid.out')
    $invalidRc = $LASTEXITCODE
    if ($invalidRc -eq 0) { Fail 'invalid model was accepted' }
    if ((Get-Item (Join-Path $work 'invalid.out')).Length -eq 0) {
        Fail 'invalid model produced no diagnostic'
    }
    Write-Host "smoke: invalid model rejected (exit $invalidRc)"

    & $Cli parse --ast --format json $Valid > (Join-Path $work 'ast.json')
    if ($LASTEXITCODE -ne 0) { Fail 'AST JSON parse failed' }
    $ast = Get-Content -Raw (Join-Path $work 'ast.json') | ConvertFrom-Json
    if (-not $ast.declarations -or $ast.declarations.Count -eq 0) {
        Fail 'AST has no declarations'
    }
    Write-Host "smoke: AST JSON parsed ($($ast.declarations.Count) declarations)"

    $pyOut = Join-Path $work 'python'
    & $Cli project --format domain-python $Valid $pyOut *> (Join-Path $work 'project.out')
    if ($LASTEXITCODE -ne 0) { Fail 'domain-python projection failed' }
    foreach ($required in @('pyproject.toml', 'README.md')) {
        if (-not (Test-Path (Join-Path $pyOut $required))) {
            Fail "domain-python projection omitted $required"
        }
    }
    if (-not (Get-ChildItem -Path (Join-Path $pyOut 'src') -Recurse -Filter 'entities.py' -File -ErrorAction SilentlyContinue)) {
        Fail 'domain-python projection omitted entities.py'
    }
    if (-not (Get-ChildItem -Path (Join-Path $pyOut 'tests') -Recurse -Filter 'test_*.py' -File -ErrorAction SilentlyContinue)) {
        Fail 'domain-python projection omitted tests'
    }
    & $python.Source -m compileall -q $pyOut
    if ($LASTEXITCODE -ne 0) { Fail 'generated Python does not compile' }
    Write-Host 'smoke: domain-python projection materialized and compiles'

    & $Cli project --format domain-python $Invalid (Join-Path $work 'bad') *> (Join-Path $work 'bad.out')
    if ($LASTEXITCODE -eq 0) { Fail 'projection accepted an invalid model' }
    Write-Host "smoke: invalid projection rejected (exit $LASTEXITCODE)"

    $rdfA = Join-Path $work 'rdf-a'
    $rdfB = Join-Path $work 'rdf-b'
    & $Cli project --format rdf --created-at 2026-01-01T00:00:00Z $Valid $rdfA *> $null
    if ($LASTEXITCODE -ne 0) { Fail 'first RDF projection failed' }
    & $Cli project --format rdf --created-at 2026-01-01T00:00:00Z $Valid $rdfB *> $null
    if ($LASTEXITCODE -ne 0) { Fail 'second RDF projection failed' }
    if (-not (Test-Path (Join-Path $rdfA 'model.ttl'))) {
        Fail 'RDF projection omitted model.ttl'
    }
    $diff = Compare-Object (Get-DirHashes $rdfA) (Get-DirHashes $rdfB)
    if ($diff) { Fail 'RDF projection is not deterministic for a fixed created-at' }
    Write-Host 'smoke: RDF projection is byte-deterministic'

    Write-Host 'smoke: PASS'
}
finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
