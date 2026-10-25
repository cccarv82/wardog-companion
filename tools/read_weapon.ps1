$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$temporary = $null
$bitmap = $null
$graphics = $null
$stream = $null
try {
    Add-Type -AssemblyName System.Runtime.WindowsRuntime
    Add-Type -AssemblyName System.Drawing
    $null = [Windows.Storage.StorageFile, Windows.Storage, ContentType=WindowsRuntime]
    $null = [Windows.Storage.Streams.IRandomAccessStream, Windows.Storage.Streams, ContentType=WindowsRuntime]
    $null = [Windows.Graphics.Imaging.BitmapDecoder, Windows.Graphics.Imaging, ContentType=WindowsRuntime]
    $null = [Windows.Graphics.Imaging.SoftwareBitmap, Windows.Graphics.Imaging, ContentType=WindowsRuntime]
    $null = [Windows.Media.Ocr.OcrEngine, Windows.Foundation, ContentType=WindowsRuntime]
    $null = [Windows.Media.Ocr.OcrResult, Windows.Foundation, ContentType=WindowsRuntime]
    $null = [Windows.Globalization.Language, Windows.Globalization, ContentType=WindowsRuntime]
    function Await($operation, [Type]$resultType) {
        $method = [System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.IsGenericMethod -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' } | Select-Object -First 1
        $task = $method.MakeGenericMethod($resultType).Invoke($null, @($operation))
        try { if (-not $task.Wait(20000)) { throw 'Windows OCR operation timed out.' } } catch { if ($task.Exception) { throw $task.Exception.GetBaseException() }; throw }
        return $task.Result
    }
    $imagePath = $env:WARDOG_OCR_TEST_IMAGE
    if (-not $imagePath) {
        Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class WardogCapture {
 [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
 [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
 [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr window, out RECT rect);
 [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr window, ref POINT point);
 [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
}
'@
        $null = [WardogCapture]::SetProcessDPIAware()
        $window = [WardogCapture]::GetForegroundWindow()
        [uint32]$gameProcess = 0
        $null = [WardogCapture]::GetWindowThreadProcessId($window, [ref]$gameProcess)
        $game = Get-Process -Id $gameProcess
        if ($game.ProcessName -ne 'WardogsClient-Win64-Shipping') { throw 'Focus the WARDOGS arsenal, then press Ctrl + Shift + R. No image was captured.' }
        $rect = New-Object WardogCapture+RECT
        $origin = New-Object WardogCapture+POINT
        if (-not [WardogCapture]::GetClientRect($window,[ref]$rect) -or -not [WardogCapture]::ClientToScreen($window,[ref]$origin)) { throw 'Cannot locate the game window.' }
        $width = $rect.Right - $rect.Left
        $height = $rect.Bottom - $rect.Top
        if ($width -lt 100 -or $height -lt 100 -or $width -gt 16000 -or $height -gt 16000) { throw 'Game window dimensions are unsupported.' }
        $bitmap = New-Object System.Drawing.Bitmap($width,$height)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($origin.X,$origin.Y,0,0,$bitmap.Size)
        $temporary = $env:WARDOG_OCR_CAPTURE_PATH
        if (-not $temporary) { throw 'Capture output was not initialized.' }
        $bitmap.Save($temporary,[System.Drawing.Imaging.ImageFormat]::Png)
        $imagePath = $temporary
    }
    $file = Await ([Windows.Storage.StorageFile]::GetFileFromPathAsync($imagePath)) ([Windows.Storage.StorageFile])
    $stream = Await ($file.OpenAsync([Windows.Storage.FileAccessMode]::Read)) ([Windows.Storage.Streams.IRandomAccessStream])
    $decoder = Await ([Windows.Graphics.Imaging.BitmapDecoder]::CreateAsync($stream)) ([Windows.Graphics.Imaging.BitmapDecoder])
    $null = [Windows.Graphics.Imaging.BitmapTransform, Windows.Graphics.Imaging, ContentType=WindowsRuntime]
    $transform = [Windows.Graphics.Imaging.BitmapTransform]::new()
    $limit = [Windows.Media.Ocr.OcrEngine]::MaxImageDimension
    $scale = [Math]::Min(1.0, $limit / [double][Math]::Max($decoder.PixelWidth,$decoder.PixelHeight))
    $transform.ScaledWidth = [uint32][Math]::Max(1,[Math]::Floor($decoder.PixelWidth * $scale))
    $transform.ScaledHeight = [uint32][Math]::Max(1,[Math]::Floor($decoder.PixelHeight * $scale))
    $software = Await ($decoder.GetSoftwareBitmapAsync([Windows.Graphics.Imaging.BitmapPixelFormat]::Bgra8,[Windows.Graphics.Imaging.BitmapAlphaMode]::Ignore,$transform,[Windows.Graphics.Imaging.ExifOrientationMode]::IgnoreExifOrientation,[Windows.Graphics.Imaging.ColorManagementMode]::DoNotColorManage)) ([Windows.Graphics.Imaging.SoftwareBitmap])
    $english = [Windows.Globalization.Language]::new('en-US')
    $engine = $null
    if ([Windows.Media.Ocr.OcrEngine]::IsLanguageSupported($english)) { $engine = [Windows.Media.Ocr.OcrEngine]::TryCreateFromLanguage($english) }
    if (-not $engine) { $engine = [Windows.Media.Ocr.OcrEngine]::TryCreateFromUserProfileLanguages() }
    if (-not $engine) { throw 'No Windows OCR language is installed. Install an OCR language in Windows Settings or select your weapon manually.' }
    if ($software.PixelWidth -gt [Windows.Media.Ocr.OcrEngine]::MaxImageDimension -or $software.PixelHeight -gt [Windows.Media.Ocr.OcrEngine]::MaxImageDimension) { throw 'Image exceeds the Windows OCR limit. Try a lower game resolution.' }
    $result = Await ($engine.RecognizeAsync($software)) ([Windows.Media.Ocr.OcrResult])
    @{ text = (($result.Lines | ForEach-Object { $_.Text }) -join "`n") } | ConvertTo-Json -Compress
} catch {
    [Console]::Error.WriteLine($_.Exception.GetBaseException().Message)
    exit 1
} finally {
    if ($stream) { $stream.Dispose() }
    if ($software) { $software.Dispose() }
    if ($graphics) { $graphics.Dispose() }
    if ($bitmap) { $bitmap.Dispose() }
    if ($temporary -and (Test-Path -LiteralPath $temporary)) { Remove-Item -LiteralPath $temporary }
}
