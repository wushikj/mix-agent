$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if($isAdmin -eq $false){
    Write-Host "please use administrator mode run this script" -ForegroundColor Yellow
    return
}

Write-Host '------------------------------------------------------------------------------------------'
Write-Host 'Copyright @2020 for Wushi (Fujian) Technology Co., Ltd'
Write-Host 'This shell script will stop all mix agent service'
Write-Host 'Current working directory is:' $(pwd)
Write-Host '------------------------------------------------------------------------------------------'

Get-Service | Where-Object {$_.Name.StartsWith("mix_agent")} | foreach-object {
    $sn = $_.ServiceName
    Stop-Service $sn
    Write-Host "Service $sn is stopped" -ForegroundColor Green
}