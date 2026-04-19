# test-receive.ps1 — Receiver script for Unicode Path Bridge testing.
# Logs the received path argument to a file so you can verify it was passed correctly.
#
# Usage (called by ubp.exe):
#   ubp.exe powershell.exe -File tests\test-receive.ps1 "C:\Some Unicode Path\file.txt"

param(
    [Parameter(Position = 0)]
    [string]$Path
)

$logFile = Join-Path $PSScriptRoot "received-path.log"

# Write with UTF-8 BOM so Notepad shows Unicode correctly
$Path | Out-File -FilePath $logFile -Encoding UTF8

# Also write to a second file with hex dump for verification
$hexFile = Join-Path $PSScriptRoot "received-path-hex.log"
$bytes = [System.Text.Encoding]::Unicode.GetBytes($Path)
$hexDump = ($bytes | ForEach-Object { '{0:X2}' -f $_ }) -join ' '
@"
Path: $Path
Length: $($Path.Length) characters
Hex (UTF-16LE): $hexDump
"@ | Out-File -FilePath $hexFile -Encoding UTF8
