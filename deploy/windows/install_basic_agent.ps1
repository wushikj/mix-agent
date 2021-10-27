$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if($isAdmin -eq $false){
    Write-Host "please use administrator mode run this script" -ForegroundColor Yellow
    return
}

# Print script introduction information
Write-Host '------------------------------------------------------------------------------------------'
Write-Host 'Copyright @2020 for Wushi (Fujian) Technology Co., Ltd'
Write-Host 'This shell script will kill all mix agent'
Write-Host 'Current working directory is:' $(pwd)
Write-Host '------------------------------------------------------------------------------------------'

function installAndStart($name) {
    $path = "$pwd\$name.exe"
    Write-Host  "Install $name"
    .\shawl.exe add --no-log --no-log-cmd --name "$name" -- $path
    Write-Host "Service $name is installed" -ForegroundColor Green
    Set-service $name -StartupType Automatic
    Start-service "$name"
    Write-Host "Service $name is started" -ForegroundColor Green
}

# install basic agent: cpu/memory/disk/machine
$mix_agent_machine ='mix_agent_machine'
installAndStart -name $mix_agent_machine

Start-Sleep –s 5

$mix_agent_cpu ='mix_agent_cpu'
installAndStart -name $mix_agent_cpu

$mix_agent_memory ='mix_agent_memory'
installAndStart -name $mix_agent_memory

$mix_agent_disk ='mix_agent_disk'
installAndStart -name $mix_agent_disk

# view install result
Get-Service | Where-Object {$_.Name.StartsWith("mix_agent")}

