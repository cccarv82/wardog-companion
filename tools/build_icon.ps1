Add-Type -AssemblyName System.Drawing
$root=Join-Path $PSScriptRoot '..\assets'
$bitmap=[Drawing.Bitmap]::new(256,256)
$g=[Drawing.Graphics]::FromImage($bitmap)
$g.SmoothingMode=[Drawing.Drawing2D.SmoothingMode]::AntiAlias
function Polygon($color,$coords) {
 $points=[Drawing.PointF[]]@(); for($i=0;$i -lt $coords.Count;$i+=2){$points += [Drawing.PointF]::new($coords[$i],$coords[$i+1])}
 $brush=[Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml($color)); $g.FillPolygon($brush,$points); $brush.Dispose(); return ,$points
}
$shield=Polygon '#11181d' @(128,12,230,48,218,158,184,204,128,244,72,204,38,158,26,48)
$pen=[Drawing.Pen]::new([Drawing.ColorTranslator]::FromHtml('#34434b'),8); $pen.LineJoin=[Drawing.Drawing2D.LineJoin]::Round; $g.DrawPolygon($pen,[Drawing.PointF[]]$shield); $pen.Dispose()
$null=Polygon '#ff6e30' @(53,67,81,77,96,143,117,98,139,98,160,143,175,77,203,67,179,177,157,187,128,130,99,187,77,177)
$null=Polygon '#ff6e30' @(109,204,128,217,147,204,128,231)
$bitmap.Save((Join-Path $root 'wardog.png'),[Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$rgba=[Collections.Generic.List[byte]]::new(); for($y=0;$y -lt 256;$y++){for($x=0;$x -lt 256;$x++){$c=$bitmap.GetPixel($x,$y);$rgba.AddRange([byte[]]@($c.R,$c.G,$c.B,$c.A))}}
[IO.File]::WriteAllBytes((Join-Path $root 'wardog.rgba'),$rgba.ToArray())
$sizes=@(16,24,32,48,64,128,256); $images=@()
foreach($size in $sizes){$scaled=[Drawing.Bitmap]::new($size,$size);$sg=[Drawing.Graphics]::FromImage($scaled);$sg.InterpolationMode=[Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic;$sg.DrawImage($bitmap,0,0,$size,$size);$ms=[IO.MemoryStream]::new();$scaled.Save($ms,[Drawing.Imaging.ImageFormat]::Png);$images+=,@($ms.ToArray());$ms.Dispose();$sg.Dispose();$scaled.Dispose()}
$file=[IO.File]::Create((Join-Path $root 'wardog.ico'));$writer=[IO.BinaryWriter]::new($file);$writer.Write([uint16]0);$writer.Write([uint16]1);$writer.Write([uint16]$sizes.Count);$offset=6+16*$sizes.Count
for($i=0;$i -lt $sizes.Count;$i++){$dimension=$sizes[$i]%256;$writer.Write([byte]$dimension);$writer.Write([byte]$dimension);$writer.Write([uint16]0);$writer.Write([uint16]1);$writer.Write([uint16]32);$writer.Write([uint32]$images[$i].Length);$writer.Write([uint32]$offset);$offset+=$images[$i].Length}
foreach($bytes in $images){$writer.Write([byte[]]$bytes)};$writer.Dispose();$bitmap.Dispose()
