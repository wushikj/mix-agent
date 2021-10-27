param([Parameter(Mandatory=$True)] [ValidateSet('install','start','restart','stop','remove','status')]$type, [string]$name='', [switch]$all)

$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if($isAdmin -eq $false){
    Write-Host "Please use administrator mode run this script" -ForegroundColor Yellow
    return
}

Write-Host '------------------------------------------------------------------------------------------'
Write-Host 'Copyright @2020 for Wushi (Fujian) Technology Co., Ltd'
Write-Host 'This shell script will install/start/restart/stop/remove/status mix agent'
Write-Host 'Current working directory is:' $(pwd)
Write-Host '------------------------------------------------------------------------------------------'

if ($all){
    if ($type -eq 'start'){
    Get-Service | Where-Object {$_.Name.StartsWith("mix_agent")} | foreach-object {
        $sn = $_.ServiceName
        Start-Service $sn
        Write-Host "Service $sn is started" -ForegroundColor Green
    }
    }

    if ($type -eq 'restart'){
        Get-Service | Where-Object {$_.Name.StartsWith("mix_agent")} | foreach-object {
            $sn = $_.ServiceName
            Restart-Service $sn
            Write-Host "Service $sn is restarted" -ForegroundColor Green
        }
    }
    if ($type -eq 'stop'){
        Get-Service | Where-Object {$_.Name.StartsWith("mix_agent")} | foreach-object {
            $sn = $_.ServiceName
            Stop-Service $sn
            Write-Host "Service $sn is stopped" -ForegroundColor Green
        }
    }

    if ($type -eq 'remove'){
        Get-Service | Where-Object {$_.Name.StartsWith("mix_agent")} | foreach-object {
            $sn = $_.ServiceName
            Stop-Service $sn
            Write-Host "Service $sn is stopped" -ForegroundColor Green
            sc.exe delete $sn
        }
    }

    Write-Host "done." -ForegroundColor Green
    return
}


$serviceName =  $name.Replace(".\","").Replace(".exe","")
Write-Host "File name is: $serviceName"
if ($serviceName -eq $null -or $serviceName -eq '')
{
    Write-Host "Please set name parameter value" -ForegroundColor Red
    return
}

Write-Host "Service name is: $serviceName"

if ($type -eq 'install')
{
    $fullPath ="$pwd\$serviceName.exe"
    Write-Host "exe full path is: $fullPath"
    $has = test-path $fullPath
    if(!$has)
    {
        Write-Host "File not found: $fullPath" -ForegroundColor Red
        return
    }

    .\shawl.exe add --no-log --no-log-cmd --name $serviceName -- $fullPath
    Write-Host "Service $serviceName is installed" -ForegroundColor Green
    Set-service $serviceName -StartupType Automatic
    Start-service  $serviceName
    Write-Host "Service $serviceName is started" -ForegroundColor Green
    return
}

if ($type -eq 'start'){
    Start-service  $serviceName
    Write-Host "Service $serviceName is started" -ForegroundColor Green
    return
}

if ($type -eq 'restart'){
    Restart-Service $serviceName
    Write-Host "Service $serviceName is restarted" -ForegroundColor Green
    return
}

if ($type -eq 'stop'){
    Stop-Service $serviceName
    Write-Host "Service $serviceName is stopped" -ForegroundColor Green
    return
}

if ($type -eq 'remove'){
    Stop-Service $serviceName
    Write-Host "service $serviceName is stopped" -ForegroundColor Green

    sc.exe delete $serviceName
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Service $serviceName is removed" -ForegroundColor Green
    }
    else {
        Write-Host "Remove  $serviceName error $_" -ForegroundColor Red
    }

    return
}

if ($type -eq 'status'){
    Get-Service $serviceName
    return
}