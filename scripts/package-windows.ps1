# Script de Empacotamento para Windows 11
# Compila os binários em modo release otimizado e prepara a pasta de distribuição.

$ErrorActionPreference = "Stop"

$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
$OutputDir = "$ProjectRoot\dist\windows"

Write-Host "=== Iniciando Empacotamento Nexa para Windows 11 ===" -ForegroundColor Cyan

# 1. Compilação Release
Write-Host "-> Compilando binários em modo release..." -ForegroundColor Yellow
Set-Location $ProjectRoot
cargo build --release --bin nexa-daemon

# 2. Prepara diretório de saída
if (Test-Path $OutputDir) {
    Remove-Item -Recurse -Force $OutputDir
}
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# 3. Copia executável e arquivos complementares
$SourceExe = "$ProjectRoot\target\release\nexa-daemon.exe"
if (-not (Test-Path $SourceExe)) {
    throw "Executável $SourceExe não encontrado!"
}

Copy-Item $SourceExe -Destination "$OutputDir\nexa-daemon.exe"
Copy-Item "$ProjectRoot\README.md" -Destination "$OutputDir\README.txt"

# 4. Cria arquivo de configuração de exemplo
$DefaultConfig = @"
# Arquivo de Configuração do Nexa
device_name = "Windows-PC"
listen_port = 24800
edge_delay_ms = 150
clipboard_sync_enabled = true
"@
$DefaultConfig | Out-File -FilePath "$OutputDir\nexa.toml" -Encoding utf8

# 5. Cria script de instalação rápida de autostart
$InstallAutostart = @"
@echo off
echo Registrando Nexa no Agendador de Tarefas do Windows...
schtasks /create /tn "NexaDaemon" /tr "\"%~dp0nexa-daemon.exe\" --daemon" /sc onlogon /rl highest /f
echo Concluido! O Nexa iniciara automaticamente no logon.
pause
"@
$InstallAutostart | Out-File -FilePath "$OutputDir\install-autostart.bat" -Encoding ascii

# 6. Compacta pacote em arquivo ZIP para distribuição
$ZipOutput = "$ProjectRoot\dist\nexa-windows-x64.zip"
Write-Host "-> Compactando em $ZipOutput..." -ForegroundColor Yellow
if (Test-Path $ZipOutput) {
    Remove-Item -Force $ZipOutput
}
Compress-Archive -Path "$OutputDir\*" -DestinationPath $ZipOutput -Force

Write-Host "=== Pacote Windows gerado com sucesso em: $OutputDir ===" -ForegroundColor Green
Write-Host "Arquivos gerados na pasta:"
Get-ChildItem $OutputDir | Select-Object Name, Length
Write-Host "Arquivo ZIP de distribuição:"
Get-Item $ZipOutput | Select-Object Name, Length
