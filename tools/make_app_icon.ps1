$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$root = Split-Path -Parent $PSScriptRoot
$logoDir = Join-Path $root "assets\logo"
$outFile = Join-Path $root "assets\icons\app.ico"
$sizes = @(16, 32, 48, 64, 80, 96, 128, 256, 512)

$sourceFiles = @(Get-ChildItem -LiteralPath $logoDir -File | Sort-Object Name)
if ($sourceFiles.Count -ne $sizes.Count) {
    throw "$logoDir contains $($sourceFiles.Count) files, expected $($sizes.Count)"
}

$sourcePaths = @{}
foreach ($file in $sourceFiles) {
    if ($file.Extension -ne ".png" -or $file.BaseName -notmatch "^RoundTable_(\d+)$") {
        throw "unexpected logo file: $($file.FullName)"
    }
    $sourceSize = [int]$Matches[1]
    if ($sourcePaths.ContainsKey($sourceSize)) {
        throw "duplicate logo size: $sourceSize"
    }
    $sourcePaths[$sourceSize] = $file.FullName
}

foreach ($size in $sizes) {
    if (-not $sourcePaths.ContainsKey($size)) {
        throw "missing logo file for ${size}x${size}"
    }
}

function New-IcoDib([System.Drawing.Bitmap]$bmp, [int]$size) {
    $rect = New-Object System.Drawing.Rectangle(0, 0, $size, $size)
    $locked = $bmp.LockBits(
        $rect,
        [System.Drawing.Imaging.ImageLockMode]::ReadOnly,
        [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
    )
    try {
        $stride = $locked.Stride
        $strideAbs = [Math]::Abs($stride)
        $pixels = New-Object byte[] ($strideAbs * $size)
        [System.Runtime.InteropServices.Marshal]::Copy(
            $locked.Scan0,
            $pixels,
            0,
            $pixels.Length
        )
    }
    finally {
        $bmp.UnlockBits($locked)
    }

    $xorBytes = $size * $size * 4
    $maskStride = [int]([Math]::Ceiling($size / 32.0) * 4)
    $maskBytes = $maskStride * $size
    $mask = New-Object byte[] $maskBytes
    $stream = New-Object System.IO.MemoryStream
    $writer = New-Object System.IO.BinaryWriter($stream)
    try {
        $writer.Write([UInt32]40)
        $writer.Write([Int32]$size)
        $writer.Write([Int32]($size * 2))
        $writer.Write([UInt16]1)
        $writer.Write([UInt16]32)
        $writer.Write([UInt32]0)
        $writer.Write([UInt32]($xorBytes + $maskBytes))
        $writer.Write([Int32]0)
        $writer.Write([Int32]0)
        $writer.Write([UInt32]0)
        $writer.Write([UInt32]0)

        for ($y = $size - 1; $y -ge 0; $y--) {
            $row = if ($stride -ge 0) { $y * $stride } else { ($size - 1 - $y) * $strideAbs }
            $writer.Write($pixels, $row, $size * 4)
            $maskRow = $size - 1 - $y
            for ($x = 0; $x -lt $size; $x++) {
                $alpha = $pixels[$row + $x * 4 + 3]
                if ($alpha -ne 0 -and $alpha -ne 255) {
                    throw "${size}x${size} source contains partial alpha"
                }
                if ($alpha -eq 0) {
                    $maskAt = $maskRow * $maskStride + [int][Math]::Floor($x / 8.0)
                    $mask[$maskAt] = [byte]($mask[$maskAt] -bor (0x80 -shr ($x % 8)))
                }
            }
        }

        $writer.Write($mask, 0, $mask.Length)
        return ,$stream.ToArray()
    }
    finally {
        $writer.Dispose()
        $stream.Dispose()
    }
}

$entries = @()
foreach ($size in $sizes) {
    $sourcePath = $sourcePaths[$size]
    $source = [System.Drawing.Bitmap]::FromFile($sourcePath)
    try {
        if ($source.Width -ne $size -or $source.Height -ne $size) {
            throw "$sourcePath is $($source.Width)x$($source.Height), expected ${size}x${size}"
        }
        if ($source.PixelFormat -ne [System.Drawing.Imaging.PixelFormat]::Format32bppArgb) {
            throw "$sourcePath is not 32bpp ARGB"
        }
        $entries += ,[pscustomobject]@{
            Size = $size
            Bytes = New-IcoDib $source $size
        }
    }
    finally {
        $source.Dispose()
    }
}

$output = New-Object System.IO.MemoryStream
$writer = New-Object System.IO.BinaryWriter($output)
try {
    $writer.Write([UInt16]0)
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]$entries.Count)

    $offset = 6 + 16 * $entries.Count
    foreach ($entry in $entries) {
        $dim = if ($entry.Size -ge 256) { 0 } else { $entry.Size }
        $writer.Write([Byte]$dim)
        $writer.Write([Byte]$dim)
        $writer.Write([Byte]0)
        $writer.Write([Byte]0)
        $writer.Write([UInt16]1)
        $writer.Write([UInt16]32)
        $writer.Write([UInt32]$entry.Bytes.Length)
        $writer.Write([UInt32]$offset)
        $offset += $entry.Bytes.Length
    }
    foreach ($entry in $entries) {
        $bytes = [byte[]]$entry.Bytes
        $writer.Write($bytes, 0, $bytes.Length)
    }

    [System.IO.File]::WriteAllBytes($outFile, $output.ToArray())
}
finally {
    $writer.Dispose()
    $output.Dispose()
}

Write-Host "wrote $outFile ($($sizes.Count) sizes: $($sizes -join ', '); 32bpp DIB)"
