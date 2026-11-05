# OS-mouse acceptance for Gumball Snappy on the real GPU window.
# Launches om9 --ui-audit with OM9_GUMBALL_OS_AUDIT, reads the ready receipt, drags the
# gumball with Win32 mouse input, then checks the committed line placement and screenshot.
param(
    [string]$OutDir = "build\gumball-os-audit",
    [string]$Exe = "L:\om9-target\debug\om9.exe"
)

$ErrorActionPreference = "Stop"
$root = (Get-Location).Path
$out = Join-Path $root $OutDir
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force $out | Out-Null

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class OsMouse {
    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X; public int Y; }
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr hwnd, ref POINT p);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hwnd, int cmd);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern bool GetCursorPos(ref POINT p);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hwnd, uint msg, UIntPtr wparam, IntPtr lparam);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint attach, uint attach_to, bool attach_flag);
    [DllImport("user32.dll")] public static extern IntPtr SetActiveWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern IntPtr SetFocus(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int cx, int cy, uint flags);
    public const uint LEFTDOWN = 0x0002;
    public const uint LEFTUP = 0x0004;
    public static readonly IntPtr TOPMOST = new IntPtr(-1);
    public const uint SWP_NOMOVE = 0x0002;
    public const uint SWP_NOSIZE = 0x0001;
    public const uint SWP_SHOWWINDOW = 0x0040;
}
"@
[void][OsMouse]::SetProcessDPIAware()

$env:OM9_GUMBALL_OS_AUDIT = $out
$project = Join-Path $out "project.om9"
& python (Join-Path $root "tools\om9_blank_project.py") $project
$deadline = (Get-Date).AddSeconds(10)
while (-not (Test-Path $project)) {
    if ((Get-Date) -gt $deadline) { throw "project fixture was not created" }
    Start-Sleep -Milliseconds 100
}
$proc = Start-Process -FilePath $Exe -ArgumentList @("--ui", $project) -PassThru

$readyPath = Join-Path $out "ready.json"
$deadline = (Get-Date).AddSeconds(30)
while (-not (Test-Path $readyPath)) {
    if ((Get-Date) -gt $deadline) { throw "ready.json was not written" }
    Start-Sleep -Milliseconds 100
}
Start-Sleep -Milliseconds 500
$ready = Get-Content $readyPath -Raw | ConvertFrom-Json
$resultPath = Join-Path $out "result.json"

$deadline = (Get-Date).AddSeconds(10)
while ($proc.MainWindowHandle -eq 0 -and (Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 100
    $proc.Refresh()
}

$proc.Refresh()
$hwnd = $proc.MainWindowHandle
if ($hwnd -eq 0) { throw "no main window handle" }
[void][OsMouse]::ShowWindow($hwnd, 9)
[void][OsMouse]::SetWindowPos($hwnd, [OsMouse]::TOPMOST, 0, 0, 0, 0, ([OsMouse]::SWP_NOMOVE -bor [OsMouse]::SWP_NOSIZE -bor [OsMouse]::SWP_SHOWWINDOW))
$fgPid = [uint32]0
$fgThread = [OsMouse]::GetWindowThreadProcessId([OsMouse]::GetForegroundWindow(), [ref]$fgPid)
$appPid = [uint32]0
$appThread = [OsMouse]::GetWindowThreadProcessId($hwnd, [ref]$appPid)
$current = [OsMouse]::GetCurrentThreadId()
[void][OsMouse]::AttachThreadInput($current, $appThread, $true)
[OsMouse]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
[OsMouse]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
[void][OsMouse]::SetForegroundWindow($hwnd)
[void][OsMouse]::BringWindowToTop($hwnd)
[void][OsMouse]::SetActiveWindow($hwnd)
[void][OsMouse]::SetFocus($hwnd)
[void][OsMouse]::AttachThreadInput($current, $appThread, $false)
Start-Sleep -Milliseconds 500

$origin = New-Object OsMouse+POINT
$origin.X = 0; $origin.Y = 0
[void][OsMouse]::ClientToScreen($hwnd, [ref]$origin)
$ppp = [double]$ready.pixels_per_point
$pivot = $ready.pivot_client
$target = $ready.target_client
$startX = [int][math]::Round($origin.X + $pivot[0] * $ppp)
$startY = [int][math]::Round($origin.Y + $pivot[1] * $ppp)
$endX = [int][math]::Round($origin.X + $target[0] * $ppp)
$endY = [int][math]::Round($origin.Y + $target[1] * $ppp)

$diagnostic = [ordered]@{
    hwnd = $hwnd.ToInt64()
    visible = [OsMouse]::IsWindowVisible($hwnd)
    foreground = [OsMouse]::GetForegroundWindow().ToInt64()
    client_origin = @($origin.X, $origin.Y)
    start = @($startX, $startY)
    end = @($endX, $endY)
}
$diagnostic | ConvertTo-Json | Set-Content (Join-Path $out "input-diagnostic.json")

Add-Type -AssemblyName System.Drawing
$rect = New-Object OsMouse+RECT
[void][OsMouse]::GetWindowRect($hwnd, [ref]$rect)
$width = $rect.Right - $rect.Left
$height = $rect.Bottom - $rect.Top
if ($width -gt 0 -and $height -gt 0) {
    $bitmap = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
    $bitmap.Save((Join-Path $out "desktop-before.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $bitmap.Dispose()
}

# Real OS pointer input after attaching the app thread to the foreground queue.
[void][OsMouse]::SetCursorPos($startX, $startY)
Start-Sleep -Milliseconds 250
[OsMouse]::mouse_event([OsMouse]::LEFTDOWN, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 80
for ($step = 1; $step -le 12; $step++) {
    $x = [int][math]::Round($startX + ($endX - $startX) * $step / 12)
    $y = [int][math]::Round($startY + ($endY - $startY) * $step / 12)
    [void][OsMouse]::SetCursorPos($x, $y)
    Start-Sleep -Milliseconds 35
}
Start-Sleep -Milliseconds 200
[OsMouse]::mouse_event([OsMouse]::LEFTUP, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 400

$deadline = (Get-Date).AddSeconds(25)
while (-not (Test-Path $resultPath)) {
    if ((Get-Date) -gt $deadline) {
        if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
        throw "result.json was not written"
    }
    Start-Sleep -Milliseconds 100
}
$result = Get-Content $resultPath -Raw | ConvertFrom-Json
$donePath = Join-Path $out "done.json"
$deadline = (Get-Date).AddSeconds(10)
while (-not (Test-Path $donePath)) {
    if ((Get-Date) -gt $deadline) { break }
    Start-Sleep -Milliseconds 100
}

if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
Remove-Item Env:OM9_GUMBALL_OS_AUDIT

$summary = [ordered]@{
    ready = $ready
    before = $result.before
    after = $result.after
    moved = $result.moved
    screenshot = (Test-Path (Join-Path $out "gumball-os-audit.png"))
    process_exited = $proc.HasExited
}
$summary | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $out "os-acceptance.json")
$summary | ConvertTo-Json -Depth 8
$done = Get-Content $donePath -Raw -ErrorAction SilentlyContinue | ConvertFrom-Json
$screenshotPath = Join-Path $out "gumball-os-audit.png"
if (-not $result.moved -or -not (Test-Path $screenshotPath) -or $done.status -ne "ok") { exit 2 }
