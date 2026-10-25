Add-Type -AssemblyName System.Drawing
$root=Join-Path $PSScriptRoot '..\installer'
$img=[Drawing.Bitmap]::new(656,1256)
$g=[Drawing.Graphics]::FromImage($img);$g.SmoothingMode=[Drawing.Drawing2D.SmoothingMode]::AntiAlias
$bg=[Drawing.Drawing2D.LinearGradientBrush]::new([Drawing.Rectangle]::new(0,0,656,1256),[Drawing.ColorTranslator]::FromHtml('#17252c'),[Drawing.ColorTranslator]::FromHtml('#090c0e'),[single]90);$g.FillRectangle($bg,0,0,656,1256)
$accent=[Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#ff6e30'));$white=[Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#edf2f5'));$muted=[Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#96a6ae'))
$g.FillRectangle($accent,60,75,72,8)
$icon=[Drawing.Image]::FromFile((Join-Path $PSScriptRoot '..\assets\wardog.png'));$g.DrawImage($icon,168,250,320,320)
$font=[Drawing.Font]::new('Segoe UI',42,[Drawing.FontStyle]::Bold,[Drawing.GraphicsUnit]::Pixel);$small=[Drawing.Font]::new('Segoe UI',24,[Drawing.FontStyle]::Regular,[Drawing.GraphicsUnit]::Pixel)
$g.DrawString('WARDOG',$font,$white,60,690);$g.DrawString('COMPANION',$font,$white,60,744)
$g.DrawString('YOUR NEXT SETUP.',$small,$muted,62,840);$g.DrawString('LESS GUESSWORK.',$small,$muted,62,880)
$g.FillRectangle($accent,60,1115,536,3);$g.DrawString('RUST  /  OPEN SOURCE',$small,$muted,60,1150)
$img.Save((Join-Path $root 'wizard.png'),[Drawing.Imaging.ImageFormat]::Png);$g.Dispose();$img.Dispose();$bg.Dispose()
$smallimg=[Drawing.Bitmap]::new(160,160);$g=[Drawing.Graphics]::FromImage($smallimg);$g.Clear([Drawing.ColorTranslator]::FromHtml('#11181d'));$g.DrawImage($icon,12,12,136,136);$smallimg.Save((Join-Path $root 'wizard-small.png'),[Drawing.Imaging.ImageFormat]::Png);$g.Dispose();$smallimg.Dispose();$icon.Dispose();$font.Dispose();$small.Dispose();$accent.Dispose();$white.Dispose();$muted.Dispose()
