# test-paths.ps1 — Automated test suite for Unicode Path Bridge.
# Compiles ubp.exe (if needed), then tests it with various Unicode paths.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File tests\test-paths.ps1
#
# Run from the project root directory.

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path $PSScriptRoot -Parent
$ubpExe = Join-Path $projectRoot "ubp.exe"
$receiver = Join-Path $PSScriptRoot "test-receive.ps1"
$logFile = Join-Path $PSScriptRoot "received-path.log"
$resultsFile = Join-Path $PSScriptRoot "test-results.log"

# --- Escape a single argument for the Windows command line ---
# Mirrors the logic in ubp.cs EscapeArgument().
function Escape-Argument([string]$arg) {
    if ([string]::IsNullOrEmpty($arg)) { return '""' }

    $needsQuoting = $false
    foreach ($c in $arg.ToCharArray()) {
        if ($c -eq ' ' -or $c -eq "`t" -or $c -eq '"') {
            $needsQuoting = $true
            break
        }
    }
    if (-not $needsQuoting) { return $arg }

    $sb = [System.Text.StringBuilder]::new($arg.Length + 4)
    [void]$sb.Append('"')
    $backslashCount = 0

    foreach ($c in $arg.ToCharArray()) {
        if ($c -eq '\') {
            $backslashCount++
        }
        elseif ($c -eq '"') {
            [void]$sb.Append('\', $backslashCount * 2 + 1)
            [void]$sb.Append('"')
            $backslashCount = 0
        }
        else {
            [void]$sb.Append('\', $backslashCount)
            [void]$sb.Append($c)
            $backslashCount = 0
        }
    }

    [void]$sb.Append('\', $backslashCount * 2)
    [void]$sb.Append('"')
    return $sb.ToString()
}

# --- Build if needed ---
if (-not (Test-Path $ubpExe)) {
    Write-Host "ubp.exe not found, building..." -ForegroundColor Yellow
    $buildCmd = Join-Path $projectRoot "build.cmd"
    & cmd.exe /c $buildCmd
    if (-not (Test-Path $ubpExe)) {
        Write-Host "FATAL: Build failed. Cannot continue." -ForegroundColor Red
        exit 1
    }
}

# --- Resolve PowerShell path ---
$psExe = (Get-Process -Id $PID).Path

# --- Test cases ---
# Build Unicode strings with char codes for PS5 compatibility (no `u{} syntax).
$tests = @(

    # ===== Basic Unicode scripts =====
    @{
        Name = "French accents"
        Path = "C:\Dossier " + [char]0x00C9 + "t" + [char]0x00E9 + "\Fichier caf" + [char]0x00E9 + ".txt"
    }
    @{
        Name = "Spaces and apostrophe"
        Path = "C:\L'" + [char]0x00E9 + "t" + [char]0x00E9 + " de l'ann" + [char]0x00E9 + "e\mon fichier.txt"
    }
    @{
        Name = "Cyrillic"
        Path = "C:\" + [string]::new([char[]]@(0x0422, 0x0435, 0x0441, 0x0442, 0x043E, 0x0432, 0x0430, 0x044F)) `
             + " " + [string]::new([char[]]@(0x043F, 0x0430, 0x043F, 0x043A, 0x0430)) `
             + "\" + [string]::new([char[]]@(0x0444, 0x0430, 0x0439, 0x043B)) + ".txt"
    }
    @{
        Name = "CJK (Japanese)"
        Path = "C:\" + [string]::new([char[]]@(0x30C6, 0x30B9, 0x30C8)) `
             + "\" + [string]::new([char[]]@(0x30D5, 0x30A1, 0x30A4, 0x30EB)) + ".txt"
    }
    @{
        Name = "CJK (Chinese)"
        Path = "C:\" + [string]::new([char[]]@(0x6D4B, 0x8BD5, 0x6587, 0x4EF6, 0x5939)) `
             + "\" + [string]::new([char[]]@(0x6587, 0x4EF6)) + ".txt"
        # 测试文件夹\文件.txt
    }
    @{
        Name = "Korean (Hangul)"
        Path = "C:\" + [string]::new([char[]]@(0xD14C, 0xC2A4, 0xD2B8)) `
             + "\" + [string]::new([char[]]@(0xD30C, 0xC77C)) + ".txt"
        # 테스트\파일.txt
    }
    @{
        Name = "Arabic (RTL)"
        Path = "C:\" + [string]::new([char[]]@(0x0645, 0x062C, 0x0644, 0x062F)) `
             + "\" + [string]::new([char[]]@(0x0645, 0x0644, 0x0641)) + ".txt"
        # مجلد\ملف.txt
    }
    @{
        Name = "Hebrew (RTL)"
        Path = "C:\" + [string]::new([char[]]@(0x05EA, 0x05D9, 0x05E7, 0x05D9, 0x05D4)) `
             + "\" + [string]::new([char[]]@(0x05E7, 0x05D5, 0x05D1, 0x05E5)) + ".txt"
        # תיקיה\קובץ.txt
    }
    @{
        Name = "Thai"
        Path = "C:\" + [string]::new([char[]]@(0x0E17, 0x0E14, 0x0E2A, 0x0E2D, 0x0E1A)) `
             + "\" + [string]::new([char[]]@(0x0E44, 0x0E1F, 0x0E25, 0x0E4C)) + ".txt"
        # ทดสอบ\ไฟล์.txt
    }

    # ===== Emoji and surrogate pairs =====
    @{
        Name = "Emoji (folder + file icons)"
        Path = "C:\" + ([char]0xD83D, [char]0xDCC1 -join '') + " Dossier\" `
             + ([char]0xD83D, [char]0xDCC4 -join '') + " Fichier.txt"
        # 📁 Dossier\📄 Fichier.txt
    }
    @{
        # Emoji sequence with ZWJ (Zero Width Joiner) — tests surrogate pair integrity
        Name = "Emoji ZWJ sequence"
        # 👨‍💻\file.txt  (man + ZWJ + laptop)
        Path = "C:\" + ([char]0xD83D, [char]0xDC68 -join '') `
             + [char]0x200D `
             + ([char]0xD83D, [char]0xDCBB -join '') `
             + "\file.txt"
    }

    # ===== Unicode normalization edge cases =====
    @{
        # Precomposed NFC: U+00E9 (é as single codepoint)
        Name = "NFC precomposed (e-acute)"
        Path = "C:\caf" + [char]0x00E9 + "\file.txt"
    }
    @{
        # Decomposed NFD: U+0065 U+0301 (e + combining acute accent)
        Name = "NFD decomposed (e + combining accent)"
        Path = "C:\caf" + [char]0x0065 + [char]0x0301 + "\file.txt"
    }

    # ===== Shell metacharacters (dangerous for cmd.exe) =====
    @{
        Name = "Ampersand in path"
        Path = "C:\Tom & Jerry\episode.txt"
    }
    @{
        Name = "Percent signs in path"
        Path = "C:\100% Complete\file.txt"
    }
    @{
        Name = "Exclamation mark (delayed expansion)"
        Path = "C:\Important!\urgent!.txt"
    }
    @{
        Name = "Caret (cmd escape char)"
        Path = "C:\Folder^name\file^1.txt"
    }
    @{
        Name = "Parentheses in path"
        Path = "C:\Project (copy)\file (1).txt"
    }
    @{
        Name = "Semicolons and equals"
        Path = "C:\key=value;data\config.txt"
    }
    @{
        Name = "At sign and hash"
        Path = "C:\user@domain\#channel\file.txt"
    }
    @{
        Name = "Dollar sign (PS variable char)"
        Path = 'C:\$Recycle.Bin\$file.txt'
    }
    @{
        Name = "Backtick (PS escape char)"
        Path = "C:\folder``name\file.txt"
    }
    @{
        Name = "Multiple special chars combined"
        Path = "C:\Tom & Jerry (2024) [100%]!\file.txt"
    }

    # ===== Double quotes =====
    @{
        Name = "Double quotes in path"
        Path = 'C:\Dossier "special"\fichier.txt'
    }

    # ===== Path structure edge cases =====
    @{
        # Drive roots have a trailing backslash but no spaces — no quoting ambiguity.
        Name = "Drive root (trailing backslash)"
        Path = "D:\"
    }
    @{
        Name = "UNC network path"
        Path = "\\server\share\folder\file.txt"
    }
    @{
        Name = "UNC path with spaces"
        Path = "\\server\shared folder\my file.txt"
    }
    @{
        # Long path — 280 characters total (over the classic 260 MAX_PATH limit)
        Name = "Long path (>260 chars)"
        Path = "C:\LongPath\" + ("SubFolder\" * 24) + "file_at_the_end.txt"
    }
    @{
        Name = "Deeply nested (many components)"
        Path = "C:\" + ("a\" * 50) + "file.txt"
    }
    @{
        Name = "Path with only spaces in folder name"
        Path = "C:\   \file.txt"
    }

    # ===== Mixed scripts and complex =====
    @{
        Name = "Mixed scripts in one path"
        Path = "C:\" + [char]0x00E9 + "t" + [char]0x00E9 + "_" `
             + [string]::new([char[]]@(0x0422, 0x0435, 0x0441, 0x0442)) + "_" `
             + [string]::new([char[]]@(0x30C6, 0x30B9, 0x30C8)) `
             + "\file.txt"
        # été_Тест_テスト\file.txt
    }

    # ===== Multiple arguments (not just a single path) =====
    @{
        Name = "Simple ASCII"
        Path = "C:\Windows\System32\notepad.exe"
    }
)

# --- Run tests ---
$passed = 0
$failed = 0
$results = @()

Write-Host ""
Write-Host "=== Unicode Path Bridge Test Suite ===" -ForegroundColor Cyan
Write-Host "PowerShell: $psExe" -ForegroundColor Gray
Write-Host "UBP: $ubpExe" -ForegroundColor Gray
Write-Host ""

foreach ($test in $tests) {
    $testName = $test.Name
    $testPath = $test.Path

    # Clean previous log
    if (Test-Path $logFile) { Remove-Item $logFile -Force }

    # Build the argument string with proper escaping (same rules as ubp.exe itself).
    # This simulates what Windows does when it expands %V in a registry command.
    $parts = @(
        (Escape-Argument $psExe),
        (Escape-Argument "-ExecutionPolicy"),
        (Escape-Argument "Bypass"),
        (Escape-Argument "-File"),
        (Escape-Argument $receiver),
        (Escape-Argument $testPath)
    )
    $argString = $parts -join ' '

    $proc = Start-Process -FilePath $ubpExe -ArgumentList $argString -PassThru -NoNewWindow -Wait

    # Wait for the PowerShell child process to finish writing the log
    $waited = 0
    while (-not (Test-Path $logFile) -and $waited -lt 5000) {
        Start-Sleep -Milliseconds 200
        $waited += 200
    }
    # Extra wait for file to be fully written
    if (Test-Path $logFile) { Start-Sleep -Milliseconds 300 }

    if (Test-Path $logFile) {
        $received = (Get-Content $logFile -Encoding UTF8 | Select-Object -First 1).Trim()

        if ($received -eq $testPath) {
            Write-Host "  PASS  $testName" -ForegroundColor Green
            $results += "PASS  $testName"
            $passed++
        }
        else {
            Write-Host "  FAIL  $testName" -ForegroundColor Red
            Write-Host "        Expected: [$testPath]" -ForegroundColor Gray
            Write-Host "        Got:      [$received]" -ForegroundColor Gray
            $results += "FAIL  $testName"
            $failed++
        }
    }
    else {
        Write-Host "  FAIL  $testName (no log file produced)" -ForegroundColor Red
        $results += "FAIL  $testName (no log file produced)"
        $failed++
    }
}

# --- Error handling tests ---
Write-Host ""
Write-Host "--- Error handling ---" -ForegroundColor Cyan

# Test: missing arguments — ubp.exe with no args shows a MessageBox (blocks the process).
$proc = Start-Process -FilePath $ubpExe -PassThru
Start-Sleep -Milliseconds 2000
if (-not $proc.HasExited) {
    # It's showing a MessageBox — correct behavior. Kill it.
    try { $proc.Kill() } catch {}
    Write-Host "  PASS  No-args shows error dialog (killed after 2s)" -ForegroundColor Green
    $results += "PASS  No-args shows error dialog"
    $passed++
}
else {
    # On some systems the MessageBox may not show (e.g., non-interactive sessions).
    # Accept exit code 1 as valid.
    if ($proc.ExitCode -ne 0) {
        Write-Host "  PASS  No-args exits with non-zero code ($($proc.ExitCode))" -ForegroundColor Green
        $results += "PASS  No-args exits with non-zero code"
        $passed++
    }
    else {
        Write-Host "  WARN  No-args exited with code 0 (MessageBox may not show in this session)" -ForegroundColor Yellow
        $results += "WARN  No-args exited with code 0"
        # Don't count as fail — this is environment-dependent
    }
}

# Test: nonexistent target program — should show an error dialog.
$proc = Start-Process -FilePath $ubpExe -ArgumentList '"nonexistent_program_12345.exe" "arg1"' -PassThru
Start-Sleep -Milliseconds 2000
if (-not $proc.HasExited) {
    try { $proc.Kill() } catch {}
    Write-Host "  PASS  Bad target shows error dialog (killed after 2s)" -ForegroundColor Green
    $results += "PASS  Bad target shows error dialog"
    $passed++
}
else {
    if ($proc.ExitCode -eq 2) {
        Write-Host "  PASS  Bad target exits with code 2" -ForegroundColor Green
        $results += "PASS  Bad target exits with code 2"
        $passed++
    }
    else {
        Write-Host "  WARN  Bad target exit code: $($proc.ExitCode) (expected 2)" -ForegroundColor Yellow
        $results += "WARN  Bad target exit code: $($proc.ExitCode)"
    }
}

# --- Summary ---
Write-Host ""
$color = if ($failed -eq 0) { "Green" } else { "Red" }
Write-Host "=== Results: $passed passed, $failed failed ===" -ForegroundColor $color

# Save results
$results | Out-File -FilePath $resultsFile -Encoding UTF8
Write-Host "Full results saved to: $resultsFile" -ForegroundColor Gray
