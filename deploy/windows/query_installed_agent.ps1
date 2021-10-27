$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if($isAdmin -eq $false){
    Write-Host "Please use administrator mode run this script" -ForegroundColor Yellow
    return
}

Write-Host '------------------------------------------------------------------------------------------'
Write-Host 'Copyright @2020 for Wushi (Fujian) Technology Co., Ltd'
Write-Host 'This shell script will query installed mix agent'
Write-Host 'Current working directory is:' $(pwd)
Write-Host '------------------------------------------------------------------------------------------'

# view install result
$service = Get-Service | Where-Object {$_.Name.StartsWith("mix_agent")}
if ($service.length -eq 0){
    Write-Host "No any agent install"
}else{
    # view install result
    Get-Service | Where-Object {$_.Name.StartsWith("mix_agent")}
}
