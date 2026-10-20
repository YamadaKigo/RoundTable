Add-Type -AssemblyName System.Drawing

$out = Join-Path $PSScriptRoot '..\assets\symbols'
New-Item -ItemType Directory -Force -Path $out | Out-Null

$magenta = [System.Drawing.Color]::FromArgb(255, 255, 0, 255)
$ink = [System.Drawing.Color]::FromArgb(255, 0, 0, 0)

function New-Symbol([string]$name, [scriptblock]$draw) {
    $bmp = [System.Drawing.Bitmap]::new(16, 16)
    for ($y = 0; $y -lt 16; $y++) {
        for ($x = 0; $x -lt 16; $x++) {
            $bmp.SetPixel($x, $y, $magenta)
        }
    }
    & $draw $bmp
    $bmp.Save((Join-Path $out "$name.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

function Set-Ink([System.Drawing.Bitmap]$bmp, [int[]]$points) {
    for ($i = 0; $i -lt $points.Count; $i += 2) {
        $bmp.SetPixel($points[$i], $points[$i + 1], $ink)
    }
}

New-Symbol 'minimize' {
    param($bmp)
    Set-Ink $bmp (4..11 | ForEach-Object { @($_, 8) })
}

New-Symbol 'maximize' {
    param($bmp)
    $points = @()
    $points += (4..11 | ForEach-Object { @($_, 4) })
    $points += (4..11 | ForEach-Object { @($_, 11) })
    $points += (4..11 | ForEach-Object { @(4, $_) })
    $points += (4..11 | ForEach-Object { @(11, $_) })
    Set-Ink $bmp $points
}

New-Symbol 'restore' {
    param($bmp)
    $points = @()
    $points += (6..11 | ForEach-Object { @($_, 4) })
    $points += (6..11 | ForEach-Object { @($_, 9) })
    $points += (6..9 | ForEach-Object { @(6, $_) })
    $points += (6..9 | ForEach-Object { @(11, $_) })
    $points += (4..9 | ForEach-Object { @($_, 6) })
    $points += (4..9 | ForEach-Object { @($_, 11) })
    $points += (6..11 | ForEach-Object { @(4, $_) })
    $points += (6..11 | ForEach-Object { @(9, $_) })
    Set-Ink $bmp $points
}

New-Symbol 'close' {
    param($bmp)
    $points = @()
    for ($v = 4; $v -le 11; $v++) {
        $points += $v
        $points += $v
        $points += $v
        $points += 15 - $v
    }
    Set-Ink $bmp $points
}

New-Symbol 'pin' {
    param($bmp)
    $points = @(
        6,4, 7,4, 8,4, 9,4,
        5,5, 6,5, 7,5, 8,5, 9,5, 10,5,
        6,6, 7,6, 8,6, 9,6,
        7,7, 8,7,
        7,8, 8,8,
        7,9, 8,9,
        6,10, 7,10, 8,10, 9,10,
        7,11, 8,11
    )
    Set-Ink $bmp $points
}

New-Symbol 'unpin' {
    param($bmp)
    $points = @(
        6,4, 7,4, 8,4, 9,4,
        5,5, 6,5, 9,5, 10,5,
        6,6, 9,6,
        7,7, 8,7,
        7,8, 8,8,
        7,9, 8,9,
        6,10, 7,10, 8,10, 9,10,
        7,11, 8,11,
        4,11, 5,10, 10,5, 11,4
    )
    Set-Ink $bmp $points
}

New-Symbol 'exclusive' {
    param($bmp)
    $points = @()
    $points += (4..11 | ForEach-Object { @($_, 4) })
    $points += (4..11 | ForEach-Object { @($_, 11) })
    $points += (4..11 | ForEach-Object { @(4, $_) })
    $points += (4..11 | ForEach-Object { @(11, $_) })
    Set-Ink $bmp $points
}

New-Symbol 'exclusive_on' {
    param($bmp)
    for ($y = 4; $y -le 11; $y++) {
        for ($x = 4; $x -le 11; $x++) {
            $bmp.SetPixel($x, $y, $ink)
        }
    }
}

New-Symbol 'page_up' {
    param($bmp)
    $points = @(
        7,3, 8,3,
        6,4, 7,4, 8,4, 9,4,
        5,5, 6,5, 7,5, 8,5, 9,5, 10,5,
        4,6, 5,6, 6,6, 7,6, 8,6, 9,6, 10,6, 11,6,
        7,7, 8,7,
        7,8, 8,8,
        7,9, 8,9,
        7,10, 8,10,
        7,11, 8,11,
        7,12, 8,12
    )
    Set-Ink $bmp $points
}

New-Symbol 'page_down' {
    param($bmp)
    $points = @(
        7,3, 8,3,
        7,4, 8,4,
        7,5, 8,5,
        7,6, 8,6,
        7,7, 8,7,
        7,8, 8,8,
        4,9, 5,9, 6,9, 7,9, 8,9, 9,9, 10,9, 11,9,
        5,10, 6,10, 7,10, 8,10, 9,10, 10,10,
        6,11, 7,11, 8,11, 9,11,
        7,12, 8,12
    )
    Set-Ink $bmp $points
}
